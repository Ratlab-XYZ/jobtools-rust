use reqwest::blocking::Client;
use serde::Deserialize;

const URL: &str = "https://data.brreg.no/enhetsregisteret/api/enheter/";

#[derive(Debug, Deserialize)]
struct TunnelResponse {
    orgnr: String,
    can_receive_ehf: Option<i32>,
    org_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PeppolResponse {
    matches: Option<Vec<PeppolMatch>>,
}

#[derive(Debug, Deserialize)]
struct PeppolMatch {
    entities: Option<Vec<PeppolEntity>>,
}

#[derive(Debug, Deserialize)]
struct PeppolEntity {
    name: Vec<PeppolName>,
}

#[derive(Debug, Deserialize)]
struct PeppolName {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OrgResponse {
    navn: Option<String>,
    organisasjonsnummer: Option<String>,
    postadresse: Option<Address>,
    forretningsadresse: Address,
    konkurs: Option<bool>,
    under_avvikling: Option<bool>,
    under_tvangsavvikling_eller_tvangsopplosning: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Address {
    adresse: Vec<String>,
    postnummer: Option<String>,
    poststed: Option<String>,
}

pub fn get_ehf(org_number: &str) -> Result<(String, String, bool), Box<dyn std::error::Error>> {
    let client = Client::new();

    let tunnel_url = format!("https://elma.ratlab.xyz/{}", org_number);
    let peppol_url = format!("https://directory.peppol.eu/search/1.0/json?q={}", org_number);

    // Try elma tunnel first
    if let Ok(response) = client
        .get(&tunnel_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
    {
        if response.status().is_success() {
            let text = response.text()?;
            let data: TunnelResponse = serde_json::from_str(&text)?;
            let ehf = data.can_receive_ehf == Some(1);
            return Ok((
                data.orgnr,
                data.org_name.unwrap_or_else(|| "Ukjent".to_string()),
                ehf,
            ));
        }
    }
    eprintln!("Fallback to Peppol directory...");

    let response = client
        .get(&peppol_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()?;
    let text = response.text()?;
    let peppol_response: PeppolResponse = serde_json::from_str(&text)?;

    if let Some(matches) = peppol_response.matches {
        if let Some(first_match) = matches.first() {
            if let Some(entities) = &first_match.entities {
                if let Some(first_entity) = entities.first() {
                    if let Some(name) = first_entity.name.first() {
                        return Ok((org_number.to_string(), name.name.clone(), true));
                    }
                }
            }
        }
    }

    // Last fallback to brreg API
    let response = client
        .get(&format!("{}{}", URL, org_number))
        .send()?;
    let text = response.text()?;
    let brreg_response: OrgResponse = serde_json::from_str(&text)?;

    Ok((
        org_number.to_string(),
        brreg_response.navn.unwrap_or_else(|| "Ukjent".to_string()),
        false,
    ))
}

pub fn print_org_details(org_number: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let url = format!("{}{}", URL, org_number);
    let response = client.get(&url).send()?;
    let text = response.text()?;
    let response: OrgResponse = serde_json::from_str(&text)?;

    let (_org_nr, _name, ehf) = get_ehf(org_number)?;

    println!("Navn: {}", response.navn.unwrap_or_else(|| "Ukjent".to_string()));
    println!("Organisasjonsnummer: {}", response.organisasjonsnummer.unwrap_or_default());
    println!("EHF: {}", if ehf { "Ja" } else { "Nei" });
    println!();

    println!("Postadresse:");
    if let Some(post) = response.postadresse {
        for linje in post.adresse {
            println!("  {}", linje);
        }
        println!("  {} {}", post.postnummer.unwrap_or_default(), post.poststed.unwrap_or_default());
    }
    println!();

    println!("Forretningsadresse:");
    for linje in &response.forretningsadresse.adresse {
        println!("  {}", linje);
    }
    println!(
        "  {} {}",
        response.forretningsadresse.postnummer.clone().unwrap_or_default(),
        response.forretningsadresse.poststed.clone().unwrap_or_default()
    );
    println!();

    println!("Status:");
    println!("  Konkurs: {}", if response.konkurs.unwrap_or(false) { "Ja" } else { "Nei" });
    println!("  Under avvikling: {}", if response.under_avvikling.unwrap_or(false) { "Ja" } else { "Nei" });
    println!("  Tvangsavvikling/-oppløsning: {}", if response.under_tvangsavvikling_eller_tvangsopplosning.unwrap_or(false) { "Ja" } else { "Nei" });

    Ok(())
}

pub fn print_org_details_short(org_number: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (org_nr, name, ehf) = get_ehf(org_number)?;

    println!("Navn: {}", name);
    println!("Organisasjonsnummer: {}", org_nr);
    println!("EHF: {}", if ehf { "Ja" } else { "Nei" });

    Ok(())
}
