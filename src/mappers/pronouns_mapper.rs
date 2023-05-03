use std::collections::HashMap;


pub struct PronounsMapper {
    hash_map: HashMap<String, String>,
}

impl PronounsMapper {
    pub fn new() -> Self {
        let subjective_vec: Vec<String> = vec![
            "she".into(),
            "he".into(),
            "they".into(),
            "it".into(),
            "one".into(),
            "ae".into(),
            "ey".into(),
            "fae".into(),
            "xe".into(),
            "ze".into(),
        ];
        let objective_vec: Vec<String> = vec![
            "her".into(),
            "him".into(),
            "them".into(),
            "its".into(),
            "one's".into(),
            "aer".into(),
            "em".into(),
            "faer".into(),
            "xem".into(),
            "hir".into(),
            "zir".into(),
        ];

        // Generate pronouns combination in form of "she/her"
        let mut hash_map: HashMap<String, String> = HashMap::new();
        for subjective in subjective_vec {
            for objective in &objective_vec {
                let combination = [subjective.clone(), objective.clone()];
                let k = combination.join("-");
                let v = combination.join("/");
                hash_map.insert(k, v);
            }
        }

        PronounsMapper { hash_map }
    }

    // Get a pronoun tag string in the form of "she/her".
    pub fn to_pronouns_tag(&self, pronouns_query: &str) -> Option<String> {
        self.hash_map.get(pronouns_query).cloned()
    }
}