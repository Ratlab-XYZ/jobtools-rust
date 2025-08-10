use reqwest::blocking::{Client, Response};
use reqwest::cookie::Jar;
use regex::Regex;
use std::sync::Arc;

fn get_viewstate(client: &Client, url: &str) -> Option<String> {
    let response = client.get(url).send().ok()?;
    let text = response.text().ok()?;

    let re = Regex::new(r#"name="javax\.faces\.ViewState".*?value="(-?\d+:\d+)""#).unwrap();
    re.captures(&text).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

pub fn send_password_reset(domain: &str, email: &str) {
    let base_url = format!("https://{}.no/controlpanel/password/", domain);

    // Enable cookie persistence
    let jar = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_store(true)
        .cookie_provider(jar.clone())
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .unwrap();

    if let Some(viewstate) = get_viewstate(&client, &base_url) {
        let form_data = [
            ("retrieve-form", "retrieve-form"),
            ("retrieve-form:query", email),
            ("javax.faces.ViewState", &viewstate),
            ("javax.faces.source", "retrieve-form:j_idt35"),
            ("javax.faces.partial.event", "click"),
            ("javax.faces.partial.execute", "retrieve-form:j_idt35 retrieve-form"),
            ("javax.faces.partial.render", "retrieve-form:query retrieve-form:queryMessage retrieve-form:backMessage"),
            ("javax.faces.partial.ajax", "true"),
        ];

        let headers = [
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("Faces-Request", "partial/ajax"), // Important for JSF
            ("X-Requested-With", "XMLHttpRequest"),
        ];

        let mut req = client.post(&base_url).form(&form_data);
        for (k, v) in headers.iter() {
            req = req.header(*k, *v);
        }

        let res: Response = req.send().unwrap();

        println!("[INFO] Sent request to {}", base_url);
        println!("[INFO] Status Code: {}", res.status());
        //println!("[INFO] Response snippet: {}", res.text().unwrap_or_default().chars().take(300).collect::<String>());
    } else {
        println!("[ERROR] Failed to retrieve ViewState");
    }
}
