use error_enum::error_type;

error_type! {
    pub ColoredError
        Test "测试" {
            #[fg = "black"]
            #[bold]
            0 BlackError (u8)
                "{0} is not black.",
            #[bg = "red"]
            #[dimmed]
            1 RedError (u8, u8)
                "{0} and {1} is not red.",
            #[fg = "green"]
            #[bg = "yellow"]
            #[underline]
            #[italic]
            2 GreenYellowError
                "Code is green and yellow.",
            #[color = "blue"]
            #[blink]
            3 BlueError
                "I'm blue.",
            #[foreground = "purple"]
            #[background = "cyan"]
            #[reverse]
            4 PurpleCyanError
                "Purpule and cyan.",
            #[color = "white"]
            #[strikethrough]
            5 WhiteError { white: String }
                "All in {white}.",
        }
}

#[test]
#[cfg(feature = "colored")]
fn main() {
    println!("{}", ColoredError::BlackError(1));
    println!("{}", ColoredError::RedError(3, 4));
    println!("{}", ColoredError::GreenYellowError);
    println!("{}", ColoredError::BlackError(9));
    println!("{}", ColoredError::BlueError);
    println!("{}", ColoredError::PurpleCyanError);
    println!(
        "{}",
        ColoredError::WhiteError {
            white: "white".to_owned()
        }
    );
}
