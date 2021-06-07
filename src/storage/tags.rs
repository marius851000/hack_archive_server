use std::collections::HashSet;



#[derive(Default)]
pub struct Tags {
    tag_list: HashSet<String>
}

impl Tags {
    pub fn add_tag_if_absent(&mut self, tag: &str) {
        if !self.tag_list.contains(tag) {
            self.tag_list.insert(tag.to_string());
        }
    }

    pub fn get_tag_list(&mut self) -> &HashSet<String> {
        return &self.tag_list
    }
}