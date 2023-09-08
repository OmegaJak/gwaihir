use gwaihir_client_lib::chrono::{DateTime, Local};

pub fn nicely_formatted_datetime(datetime: DateTime<Local>) -> String {
    let time_format = "%l:%M%P";
    if datetime.date_naive() == Local::now().date_naive() {
        return datetime.format(time_format).to_string();
    } else {
        return datetime.format(&format!("%D {}", time_format)).to_string();
    }
}
