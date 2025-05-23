pub struct CliArgs {
    pub path: String,
    pub dump: bool,
}

impl CliArgs {
    pub fn parse(args: &[String]) -> anyhow::Result<CliArgs> {
        let mut cli_args = CliArgs {
            path: "".to_string(),
            dump: false,
        };

        for arg in args {
            if arg.starts_with("--") {
                match arg.as_str() {
                    "--dump" => cli_args.dump = true,
                    _ => anyhow::bail!("Unsupported argument: {arg}"),
                }
            } else {
                cli_args.path = arg.to_string();
            }
        }

        Ok(cli_args)
    }
}
