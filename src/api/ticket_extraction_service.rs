use regex::Regex;

pub trait TicketExtractionService: Sync + Send {
    fn regex(&self) -> &Regex;

    fn extract_tickets(&self, text: &str) -> Vec<String> {
        self.regex()
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

pub struct FooTicketExtractionService {
    regex: Regex,
}

pub struct BarTicketExtractionService {
    regex: Regex,
}

impl FooTicketExtractionService {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"[fF][oO][oO]-\d{1,6}").unwrap(),
        }
    }
}

impl TicketExtractionService for FooTicketExtractionService {
    fn regex(&self) -> &Regex {
        &self.regex
    }
}

impl BarTicketExtractionService {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"[bB][aA][rR]-\d{1,6}").unwrap(),
        }
    }
}

impl TicketExtractionService for BarTicketExtractionService {
    fn regex(&self) -> &Regex {
        &self.regex
    }
}
