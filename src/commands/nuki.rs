use serenity::builder::CreateApplicationCommand;

use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(_options: &[CommandDataOption]) -> String {
    "Hey, I'm alive!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("nuki")
        .description("Log your nukis")
        .create_option(|opt| {
            opt.name("amount")
                .description("The numbe of times you have nuki'd")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(20)
                .required(false)
        })
        .create_option(|opt| {
            opt.name("comment")
                .description("Optional comment")
                .kind(CommandOptionType::String)
                .required(false)
        })
}
