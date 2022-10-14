// This is a modified version of the example at:
// https://github.com/Byron/google-apis-rs/tree/main/gen/sheets4

extern crate google_sheets4 as sheets4;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
use sheets4::{Error, Sheets};

// The ID and range of a sample spreadsheet.
const SPREADSHEET_ID: &str = "1mV2Q2WVZnPTZs9VsJyFQ2utrHRg_RSAfbqYHnEfQFaY";
const RANGE_NAME: &str = "A4:O34";


#[tokio::main]
async fn main() {
    // Get an ApplicationSecret instance by some means. It contains the `client_id` and
    // `client_secret`, among other things.
    let secret = yup_oauth2::read_application_secret("credentials.json")
        .await
        .expect("client secret could not be read");

    // Instantiate the authenticator. It will choose a suitable authentication flow for you,
    // unless you replace  `None` with the desired Flow.
    // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
    // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
    // retrieve them from storage.
    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await
    .unwrap();

    let hub = Sheets::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), 
        auth);

    // let hub = Sheets::new(
    //     hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
    //     auth,
    // );

    let result = hub.spreadsheets().values_get(SPREADSHEET_ID, RANGE_NAME)
             .doit()
             .await;

    //println!("{:?}",result);
    match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => {set_schedulers(res.1.values.unwrap())},
    }

    fn set_schedulers(_res: Vec<Vec<String>>) {
        let row = ["A", "B", "C", "D", "E",	"F", "G", "H", "I", "J", "K", "L", "M" ,"N", "O", "P"];
        for (_ir, _r) in _res.iter().enumerate() {
            println!("//__________________________________ Column {0} _______________________________________//", _ir);
            for (_ic, _c) in _r.iter().enumerate() {
                print!("|{1}{2}: {0}|, ", _c, row[_ic], _ir + 4);
            }
            println!("\n\\_______________________________________________________________________________________\\");
        }
    }
}