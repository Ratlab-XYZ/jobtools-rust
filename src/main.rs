use clap::{Parser, Subcommand};

mod dns_tools;
mod resetpw;
mod org;
mod ssh;
mod cheat;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand, Debug)]
enum Commands {
    /// Find text in cheat sheet
    Cheat {
        search_word: String
    },
    /// Check DNS for domain
    Check {
        //#[arg(short, long)]
        domain: String
    },
    /// Connect to a webhotel via ssh
    Connect {
        cluster: String
    },

    /// Check if org number can recive EHF 
    Ehf {
        org_number: String
    },

    /// Get org details
    Org {
        org_number: String
    },

    /// Send reset password link to customer's email
    Resetpw {
        domain: String,
        email: String
    },

    /// Update tool
    Update {

    }

}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Check { domain } => {
             dns_tools::check_domain(&domain);
        }

        Commands::Cheat { search_word } => {
            
            let results = cheat::cheat_sheet(search_word);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            for line in results {
                println!("{}", line);
            }
        }

        Commands::Connect { cluster } => {
            ssh::ssh_connect( cluster );
        }

        Commands::Ehf { org_number } => {
            match org::print_org_details_short(org_number)  {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
            };
        }

        Commands::Org { org_number } => {
            match org::print_org_details(org_number)  {
                Ok(_) => (),
                Err(e) => eprintln!("Error: {}", e),
            };
        }

        Commands::Resetpw {domain, email} => {
            resetpw::send_password_reset(domain, email);
        }

        Commands::Update { .. } => {
            println!("Updating....")
        }
    }
}
