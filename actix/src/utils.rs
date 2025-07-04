// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::HttpRequest;
use nanoid::nanoid;
use rand::seq::IndexedRandom;
use regex::Regex;
use rusqlite::Connection;
use serde::Deserialize;
use std::iter;
use once_cell::sync::Lazy;
use rand::Rng;
use sha2::{Sha256, Digest};

use crate::{config::Config, database};

// Struct for reading link pairs sent during API call
#[derive(Deserialize)]
struct URLPair {
    shortlink: String,
    longlink: String,
}

// Struct for readling user edit API call
#[derive(Deserialize)]
struct EditLinkJson {
    longlink: String,
}

// Regex for URL
const URL_REGEX: &str = r"http[s]?:\/\/(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\(\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+";

// Lazy Statics for URL Regex
static LAZY_REGEX: Lazy<Regex>=  Lazy::new(|| {
    Regex::new(URL_REGEX).unwrap()
});

// Request the DB for searching an URL
pub fn get_longurl(shortlink: String, db: &Connection) -> Option<String> {
    if validate_link(&shortlink) {
        database::find_url(shortlink.as_str(), db)
    } else {
        None
    }
}

pub fn is_api_header(httprequest: &HttpRequest) -> bool {
    httprequest.headers().get("x-api-key").is_some()
}

// Only have a-z, 0-9, - and _ as valid characters in a shortlink
fn validate_link(link: &str) -> bool {
    let re = Regex::new("^[a-z0-9-_]+$").expect("Regex generation failed.");
    re.is_match(link)
}

// Request the DB for all URLs
pub fn getall(db: &Connection) -> String {
    let links = database::getall(db);
    serde_json::to_string(&links).expect("Failure during creation of json from db.")
}

// Make checks and then request the DB to add a new URL entry
pub fn add_link(
    req: String,
    db: &Connection,
    config: &Config
) -> (bool, String) {
    let mut chunks: URLPair;
    if let Ok(json) = serde_json::from_str(&req) {
        chunks = json;
    } else {
        // shorturl should always be supplied, even if empty
        return (false, String::from("Invalid request!"));
    }

    // Check if longlink is actually URL
    if !url_scheme_check(chunks.longlink.clone()) {
        return (
            false,
            String::from("URL scheme check failed!"),
        )
    }

    let style = config.slug_style.clone();
    let len = config.slug_length;

    if chunks.shortlink.is_empty() {
        chunks.shortlink = gen_link(style, len);
    }

    if validate_link(chunks.shortlink.as_str())
        && get_longurl(chunks.shortlink.clone(), db).is_none()
    {
        (
            database::add_link(chunks.shortlink.clone(), chunks.longlink, db),
            chunks.shortlink,
        )
    } else {
        (
            false,
            String::from("Short URL not valid or already in use!"),
        )
    }
}

// Make Check then edit the longurl link
pub fn edit_link(req: String, shortlink: String, db: &Connection) -> (bool, String) {
    let chunks: EditLinkJson;
    if let Ok(json) = serde_json::from_str(&req) {
        chunks = json;
    } else {
        // shorturl should always be supplied, even if empty
        return (false, String::from("Invalid request!"));
    }

    if shortlink.is_empty() {
        (false, String::from("Invaild edit parameter received."));
    }

    // Check if longlink is actually URL
    if !url_scheme_check(chunks.longlink.clone()) {
        return (
            false,
            String::from("URL scheme check failed!"),
        )
    }

    let db_longurl = get_longurl(shortlink.clone(), db);

    // url compares moved here
    if let Some(ref url) = db_longurl {
        if url == &chunks.longlink {
            return (
                false,
                String::from("LongURL is the same!")
            );
        } else {
            return (
                database::edit_link(shortlink.clone(), chunks.longlink, db),
                shortlink,
            );
        }
    } else {
        return (
            false,
            String::from("LongURL does not existed!")
        );
    }

}

// Check if link, and request DB to delete it if exists
pub fn delete_link(shortlink: String, db: &Connection) -> bool {
    if validate_link(shortlink.as_str()) {
        database::delete_link(shortlink, db)
    } else {
        false
    }
}

// Generate a simple API Key(totally not secured!)
pub fn gen_api_key(
    db: &Connection,
    string_size: usize,
) -> (bool, String) {
    let generated_key: String = generate_string(string_size);
    let hashed_key: String = hash_string(&generated_key);
    (database::add_api_key(hashed_key, db),
    generated_key)
}

// Reset API Key
pub fn reset_api_key(
    db: &Connection,
) -> (bool, String) {
    (database::reset_api_key(db),
    String::from("done"))
}

// Generate Random String
// From: https://stackoverflow.com/a/74953997
fn generate_string(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
    iter::repeat_with(one_char).take(len).collect()
}

// Generate a random link using either adjective-name pair (default) of a slug or a-z, 0-9
fn gen_link(style: String, len: usize) -> String {
    #[rustfmt::skip]
    static ADJECTIVES: [&str; 108] = ["admiring", "adoring", "affectionate", "agitated", "amazing", "angry", "awesome", "beautiful", 
		"blissful", "bold", "boring", "brave", "busy", "charming", "clever", "compassionate", "competent", "condescending", "confident", "cool", 
		"cranky", "crazy", "dazzling", "determined", "distracted", "dreamy", "eager", "ecstatic", "elastic", "elated", "elegant", "eloquent", "epic", 
		"exciting", "fervent", "festive", "flamboyant", "focused", "friendly", "frosty", "funny", "gallant", "gifted", "goofy", "gracious", 
		"great", "happy", "hardcore", "heuristic", "hopeful", "hungry", "infallible", "inspiring", "intelligent", "interesting", "jolly", 
		"jovial", "keen", "kind", "laughing", "loving", "lucid", "magical", "modest", "musing", "mystifying", "naughty", "nervous", "nice", 
		"nifty", "nostalgic", "objective", "optimistic", "peaceful", "pedantic", "pensive", "practical", "priceless", "quirky", "quizzical", 
		"recursing", "relaxed", "reverent", "romantic", "sad", "serene", "sharp", "silly", "sleepy", "stoic", "strange", "stupefied", "suspicious", 
		"sweet", "tender", "thirsty", "trusting", "unruffled", "upbeat", "vibrant", "vigilant", "vigorous", "wizardly", "wonderful", "xenodochial", 
		"youthful", "zealous", "zen"];
    #[rustfmt::skip]
    static NAMES: [&str; 241] = ["agnesi", "albattani", "allen", "almeida", "antonelli", "archimedes", "ardinghelli", "aryabhata", "austin", 
		"babbage", "banach", "banzai", "bardeen", "bartik", "bassi", "beaver", "bell", "benz", "bhabha", "bhaskara", "black", "blackburn", "blackwell", 
		"bohr", "booth", "borg", "bose", "bouman", "boyd", "brahmagupta", "brattain", "brown", "buck", "burnell", "cannon", "carson", "cartwright", 
		"carver", "cauchy", "cerf", "chandrasekhar", "chaplygin", "chatelet", "chatterjee", "chaum", "chebyshev", "clarke", "cohen", "colden", "cori", 
		"cray", "curie", "curran", "darwin", "davinci", "dewdney", "dhawan", "diffie", "dijkstra", "dirac", "driscoll", "dubinsky", "easley", "edison", 
		"einstein", "elbakyan", "elgamal", "elion", "ellis", "engelbart", "euclid", "euler", "faraday", "feistel", "fermat", "fermi", "feynman", "franklin", 
		"gagarin", "galileo", "galois", "ganguly", "gates", "gauss", "germain", "goldberg", "goldstine", "goldwasser", "golick", "goodall", "gould", "greider", 
		"grothendieck", "haibt", "hamilton", "hardy", "haslett", "hawking", "heisenberg", "hellman", "hermann", "herschel", "hertz", "heyrovsky", "hodgkin", 
		"hofstadter", "hoover", "hopper", "hugle", "hypatia", "ishizaka", "jackson", "jang", "jemison", "jennings", "jepsen", "johnson", "joliot", "jones", 
		"kalam", "kapitsa", "kare", "keldysh", "keller", "kepler", "khayyam", "khorana", "kilby", "kirch", "knuth", "kowalevski", "lalande", "lamarr", 
		"lamport", "leakey", "leavitt", "lederberg", "lehmann", "lewin", "lichterman", "liskov", "lovelace", "lumiere", "mahavira", "margulis", "matsumoto", 
		"maxwell", "mayer", "mccarthy", "mcclintock", "mclaren", "mclean", "mcnulty", "meitner", "mendel", "mendeleev", "meninsky", "merkle", "mestorf", 
		"mirzakhani", "montalcini", "moore", "morse", "moser", "murdock", "napier", "nash", "neumann", "newton", "nightingale", "nobel", "noether", "northcutt", 
		"noyce", "panini", "pare", "pascal", "pasteur", "payne", "perlman", "pike", "poincare", "poitras", "proskuriakova", "ptolemy", "raman", "ramanujan", 
		"rhodes", "ride", "riemann", "ritchie", "robinson", "roentgen", "rosalind", "rubin", "saha", "sammet", "sanderson", "satoshi", "shamir", "shannon", 
		"shaw", "shirley", "shockley", "shtern", "sinoussi", "snyder", "solomon", "spence", "stonebraker", "sutherland", "swanson", "swartz", "swirles", 
		"taussig", "tesla", "tharp", "thompson", "torvalds", "tu", "turing", "varahamihira", "vaughan", "vaughn", "villani", "visvesvaraya", "volhard", 
		"wescoff", "weierstrass", "wilbur", "wiles", "williams", "williamson", "wilson", "wing", "wozniak", "wright", "wu", "yalow", "yonath", "zhukovsky"];

    #[rustfmt::skip]
    static CHARS: [char; 36] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

    if style == "UID" {
        nanoid!(len, &CHARS)
    } else {
        format!(
            "{0}-{1}",
            ADJECTIVES
                .choose(&mut rand::rng())
                .expect("Error choosing random adjective."),
            NAMES
                .choose(&mut rand::rng())
                .expect("Error choosing random name.")
        )
    }
}

// Check if input is URL or not.
fn url_scheme_check(url: String) -> bool {
    return LAZY_REGEX.is_match(&url)
}

// hash a string to be kept in db(api-key)
pub fn hash_string(input: &String) -> String {
    return format!("{:x}", Sha256::digest(input));
}