pub mod wallets;

pub use self::wallets::WalletsService;

static WALLETS_SERVICE_ID: u16 = 1;
static WALLETS_SERVICE_NAME: &str = "wallets";
