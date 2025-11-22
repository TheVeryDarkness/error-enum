use error_enum::error_type;

error_type! {
    #[derive(Debug)]
    pub ColoredError
        /// 测试
        {
            #[diag(kind = "error")]
            #[diag(code = 0)]
            #[diag(msg = "SimpleError")]
            {
                #[diag(code = 0)]
                #[diag(msg = "{0} is not black.")]
                BlackError (u8),
                #[diag(code = 1)]
                #[diag(msg = "{0} and {1} is not red.")]
                RedError (u8, u8),
                #[diag(code = 2)]
                #[diag(msg = "Code is green and yellow.")]
                GreenYellowError,
                #[diag(code = 3)]
                #[diag(msg = "I'm blue.")]
                BlueError,
                #[diag(code = 4)]
                #[diag(msg = "Purpule and cyan.")]
                PurpleCyanError,
                #[diag(code = 5)]
                #[diag(msg = "All in {white}.")]
                WhiteError { white: String },
            },
        },
}
