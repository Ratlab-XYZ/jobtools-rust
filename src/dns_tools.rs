use colored::*;
use std::net::IpAddr;
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};
use terminal_size::{terminal_size, Width};

const ALLOWED_MX: &[&str] = &[
    "z.tornado.email.", "y.tornado.email.", "x.tornado.email.",
    "mx3-proisp-no.pub.mailpod1-osl1.one.com.",
    "mx2-proisp-no.pub.mailpod1-osl1.one.com.",
    "mx1-proisp-no.pub.mailpod1-osl1.one.com.",
    "mx1.pub.mailpod1-osl1.one.com.",
    "mx2.pub.mailpod1-osl1.one.com.",
    "mx3.pub.mailpod1-osl1.one.com.",
];

/// Main entry point for DNS checking
pub fn check_domain(domain: &str) {
    println!("{} {}", "Checking domain:".bold(), domain.blue());

    query_dns(domain, "A");
    query_dns(domain, "MX");
    query_dns(domain, "TXT");
    query_dns(domain, "NS");
    query_dns(domain, "SOA");

    resolve_mx_ips(domain);
}

pub fn print_header(name: &str) {
    let width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80); // Fallback if terminal size can't be read

    let prefix = format!("━━━━━━━━ {} Records ", name);
    let remaining_cols = width.saturating_sub(visible_width(&prefix));
    let line = format!("{}{}", prefix, "━".repeat(remaining_cols));


    println!("{}", line);
}

fn visible_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

fn query_dns(domain: &str, record_type: &str) {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
        .expect("Failed to create resolver");

    print_header(record_type);

    match record_type {
        "A" => {
            if let Ok(response) = resolver.ipv4_lookup(domain) {
                let results: Vec<_> = response.iter().collect();
                if !results.is_empty() {
                    for ip in results {
                        print_record(domain, record_type, ip.to_string().green());
                    }
                } else {
                    print_soa(&resolver, domain);
                }
            } else {
                print_soa(&resolver, domain);
            }
        }
        "MX" => {
            if let Ok(response) = resolver.mx_lookup(domain) {
                let results: Vec<_> = response.iter().collect();
                if !results.is_empty() {
                    for mx in results {
                        let val = format!("{} {}", mx.preference(), mx.exchange());
                        print_record(domain, record_type, val.cyan());
                    }
                } else {
                    print_soa(&resolver, domain);
                }
            } else {
                print_soa(&resolver, domain);
            }
        }
        "TXT" => {
            if let Ok(response) = resolver.txt_lookup(domain) {
                let results: Vec<_> = response.iter().collect();
                if !results.is_empty() {
                    for txt in results {
                        let joined = txt
                            .iter()
                            .map(|s| String::from_utf8_lossy(s).to_string())
                            .collect::<Vec<_>>()
                            .join(" ");
                        print_record(domain, record_type, joined.red());
                    }
                } else {
                    print_soa(&resolver, domain);
                }
            } else {
                print_soa(&resolver, domain);
            }
        }
        "NS" => {
            if let Ok(response) = resolver.ns_lookup(domain) {
                let results: Vec<_> = response.iter().collect();
                if !results.is_empty() {
                    for ns in results {
                        print_record(domain, record_type, ns.to_string().yellow());
                    }
                } else {
                    print_soa(&resolver, domain);
                }
            } else {
                print_soa(&resolver, domain);
            }
        }
        "SOA" => {
            print_soa(&resolver, domain);
        }
        _ => {}
    }
}

fn print_soa(resolver: &Resolver, domain: &str) {
    if let Ok(response) = resolver.soa_lookup(domain) {
        for soa in response {
            let val = format!(
                "{} {} {} {} {} {} {}",
                soa.mname(), soa.rname(), soa.serial(),
                soa.refresh(), soa.retry(), soa.expire(), soa.minimum()
            );
            print_record(domain, "SOA", val.dimmed().yellow());
        }
    } else {
        println!("{}", "SOA lookup failed".red());
    }
}

fn print_record(domain: &str, record_type: &str, value: colored::ColoredString) {
    println!(
        "{}\t{}\t{}\t{}\t{}",
        domain.blue(),
        "3600".green(),
        "IN".yellow(),
        record_type.magenta(),
        value
    );
}

fn resolve_mx_ips(domain: &str) {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
        .expect("Failed to create resolver");

    if let Ok(response) = resolver.mx_lookup(domain) {
        let mx_records: Vec<String> = response.iter().map(|mx| mx.exchange().to_utf8()).collect();
        let unresolved: Vec<String> = mx_records
            .into_iter()
            .filter(|mx| !ALLOWED_MX.contains(&mx.as_str()))
            .collect();

        if !unresolved.is_empty() {
            println!("{}", "Resolving IP addresses for MX records:".bold());
            for mx in unresolved {
                if let Ok(a_response) = resolver.ipv4_lookup(&mx) {
                    // FIX: Convert each `A` record to IpAddr first, then to String
                    let ips: Vec<String> = a_response
                        .iter()
                        .map(|a| IpAddr::V4(**a)) // `a` is an `A` record
                        .map(|ip| ip.to_string())
                        .collect();

                    println!("{} ⇒  {}", mx, ips.join(" "));
                }
            }
        }
    }
}
