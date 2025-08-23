pub struct Setup{
    pub port: u16,
    pub private_input: u64,
    pub random_value: u64,
}

pub static SETUP: Setup = Setup {
    port: 8081,
    private_input: 10,
    random_value: 7,
};