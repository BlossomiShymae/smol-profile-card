use std::io::{BufWriter, Cursor};
use std::sync::Arc;
use axum::body::Full;
use axum::http::{StatusCode};
use axum::extract::{State, Query};
use axum::response::{Response, IntoResponse, Html};
use image::imageops::FilterType;
use image::{Rgba, ImageFormat, DynamicImage, RgbaImage, ImageBuffer};
use serde::Deserialize;
use rusttype::{Scale, Font};

use crate::AppState;
use crate::validators;


#[derive(Debug, Deserialize)]
pub struct GithubUserViewModel {
    user: String,
    pronouns: Option<String>
}

impl GithubUserViewModel {
    pub fn is_valid(&self) -> bool {
        // Github username has a 39 character limit
        let mut is_user_valid = validators::is_str_valid_length(&self.user, 0, 39);
        is_user_valid = is_user_valid && validators::is_str_delimiter_free(&self.user);

        let is_pronouns_valid = match &self.pronouns {
            Some(pronouns) => validators::is_str_delimiter_free(&pronouns),
            None => true
        };

        is_user_valid && is_pronouns_valid
    }
}

#[axum_macros::debug_handler]
pub async fn get_index(query: Query<GithubUserViewModel>, State(state): State<Arc<AppState>>) -> Response {
    let vm = query.0;
    if !vm.is_valid() {
        return super::get_error_page(&state.registry, StatusCode::BAD_REQUEST)
            .await;
    }

    let username = vm.user.to_string();
    let pronouns = vm.pronouns;
    let pronouns_tag = match pronouns {
        Some(query) => state.pronouns_mapper
            .to_pronouns_tag(&query)
            .unwrap_or_else(|| { "".to_string() }),
        None => String::from("")
    };

    log::trace!("User: {}", username);
    log::trace!("Pronouns: {}", pronouns_tag);

    let user_result = state.github_user_service
        .get_by_username(&username)
        .await;

    if let Ok(user_option) = user_result {
        if let Some(user) = user_option {
            let avatar_result = state.github_user_service.get_avatar_by_id(user.id).await;
            if avatar_result.is_err() {
                return super::get_error_page(&state.registry, StatusCode::INTERNAL_SERVER_ERROR).await
            }

            // Load avatar image
            let avatar = avatar_result.unwrap();
            let mut avatar_img = image::load_from_memory(&avatar).unwrap();
            // Border round the avatar
            let mut canvas_avatar = avatar_img.to_rgba8();
            round_image_mut(&mut canvas_avatar);
            // Overlay avatar onto image
            avatar_img = canvas_avatar.into();
            avatar_img = avatar_img.resize(100, 100, FilterType::Lanczos3);
            let mut card_img = draw_image(&user, &pronouns_tag).await;
            image::imageops::overlay(&mut card_img, &avatar_img, 20, 10);
            // Serialize image
            let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
            card_img.write_to(&mut buffer, ImageFormat::Png).unwrap();
            let bytes: Vec<u8> = buffer.into_inner().unwrap().into_inner();
            
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "image/png")
                .header("Content-Length", bytes.len().to_string())
                .body(Full::from(bytes))
                .unwrap()
                .into_response();
        }
    }

    super::get_error_page(&state.registry, StatusCode::INTERNAL_SERVER_ERROR).await
}

pub async fn get_html(query: Query<GithubUserViewModel>, State(state): State<Arc<AppState>>) -> Response {
    let vm = query.0;
    if !vm.is_valid() {
        return super::get_error_page(&state.registry, StatusCode::BAD_REQUEST).await;
    }

    let r = format!("<img class=\"img-fluid img-thumbnail\" src=\"http://localhost:8080/image?user={}&pronouns={}\"/>",
        vm.user.as_str(),
        vm.pronouns.unwrap_or("".to_string()).as_str()
    );
    log::debug!("{}", r);
    return (StatusCode::OK, Html(r)).into_response();
}

fn round_image_mut(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let dim = image.dimensions();
    let mut canvas_mask = RgbaImage::new(dim.0, dim.1);
    let width = dim.0 as f32;
    let height = dim.1 as f32;
    let midpoint = ((width/2.0) as i32, (height/2.0) as i32);
    let radius = (width as i32) - midpoint.0;
    imageproc::drawing::draw_filled_circle_mut(
        &mut canvas_mask, 
        midpoint,
        radius,
        Rgba([255,255,255,255])
    );
    // Apply clip mask
    for (avatar_p, mask_p) in image.pixels_mut().zip(canvas_mask.pixels_mut()) {
        if mask_p.0 != [255,255,255,255] {
            avatar_p.0 = [0,0,0,0];
        }
    }
}

async fn draw_image(user: &crate::models::github_user::GithubUser, pronouns_tag: &str) -> DynamicImage {
    // Create profile card image
    let mut img = image::open("images/dark_template.png").unwrap();
    let mut location_img = image::open("images/location.png").unwrap();
    let regular_font_data = Vec::from(include_bytes!("../../fonts/Oxygen-Regular.ttf") as &[u8]);
    let regular_font = Font::try_from_vec(regular_font_data).unwrap();
    let light_font_data = Vec::from(include_bytes!("../../fonts/Oxygen-Light.ttf") as &[u8]);
    let light_font = Font::try_from_vec(light_font_data).unwrap();
    let big_font_size = 24.0;
    let smol_font_size = 20.0;
    
    // Draw the person's name
    {
        let left_margin = 140;
        let user = user.clone();
        imageproc::drawing::draw_text_mut(
            &mut img, 
            Rgba([255u8, 255u8, 255u8, 255u8]), 
            left_margin, 
            20, 
            Scale { x: big_font_size, y: big_font_size },
            &regular_font, 
            &user.name.unwrap_or(user.login.to_string())
        );
        // Draw the person's location
        let location = &user.location.unwrap_or("".to_string());
        imageproc::drawing::draw_text_mut(
            &mut img, 
            Rgba([192u8, 192u8, 192u8, 255u8]), 
            left_margin + 20, 
            50, 
            Scale {
                x: smol_font_size,
                y: smol_font_size
            }, 
            &light_font,
            location
        );
        // Overlay location icon
        if location.ne("") {
            image::imageops::invert(&mut location_img);
            let buffer = image::imageops::brighten(&location_img, -25);
            image::imageops::overlay(&mut img, &buffer, i64::from(left_margin), 55);
        }
        // Draw the person's pronouns
        imageproc::drawing::draw_text_mut(
            &mut img, 
            Rgba([192u8, 192u8, 192u8, 255u8]), 
            left_margin, 
            78, 
            Scale {
                x: smol_font_size,
                y: smol_font_size
            }, 
            &light_font, 
            pronouns_tag.clone()
        );
    };   

    img
}