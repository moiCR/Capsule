pub mod language_manager;

pub struct Language {
    pub name: String,
    pub days: Days,
}

pub struct Days {
    pub monday: String,
    pub tuesday: String,
    pub wednesday : String,
    pub thursday : String,
    pub friday : String,
    pub saturday : String,
    pub sunday : String,
}
