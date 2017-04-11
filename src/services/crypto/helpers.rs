extern crate rand;
extern crate milagro_crypto;
use self::milagro_crypto::randapi::Random;
use self::milagro_crypto::big::wrappers::MODBYTES;
use self::milagro_crypto::ff::FF;
use self::milagro_crypto::hash::wrappers::hash256;
use self::rand::os::OsRng;
use self::rand::Rng;
use services::crypto::constants::{
    BIG_SIZE,
    BN_MASK,
    PRIMES,
    NUM_PRIMES,
    LARGE_PRIME
};
use services::crypto::types::{ByteOrder};

pub fn generate_random_seed() -> [u8; 32] {
    let mut seed: [u8; 32] = [0; 32];
    let mut rng = OsRng::new().unwrap();
    rng.fill_bytes(&mut seed);
    seed
}

pub fn generate_big_random(size: usize) -> FF {
    let seed = generate_random_seed();
    let mut rng = Random::new(seed);

    let b_size: usize = size / 8 + 1; // number of bytes for mod value
    let big_size: usize = (b_size + MODBYTES - 1) / MODBYTES; // number of BIGs for mod value

    // init mod bytes with 0 and set 1 in proper place
    let mut bytes = vec![0; big_size * MODBYTES];
    bytes[big_size * MODBYTES - b_size] = (1 as u8).wrapping_shl((size - (b_size - 1) * 8) as u32);

    let bv = FF::from_bytes(&bytes[0..], big_size * MODBYTES, BIG_SIZE);
    let r = FF::randomnum(&bv, &mut rng);
    r
}

pub fn generate_prime(size: usize) -> FF {
    let seed = generate_random_seed();
    let mut rng = Random::new(seed);
    let mut iteration = 0;
    let mut is_prime = false;

    let mut prime = generate_big_random(size);
    let mut prime_bytes = prime.to_bytes();
    let length = prime_bytes.len();
    let last_byte = prime_bytes[length - 1];
    prime_bytes[length - 1] = last_byte | 3;
    prime = FF::from_bytes(&prime_bytes, length, BIG_SIZE);
    while !is_prime {
        prime.inc(4);
        iteration += 1;
        is_prime = FF::is_prime(&prime, &mut rng);
    }
    debug!("Iteration: {}\nFound prime: {}", iteration, prime);

    prime
}

pub fn generate_prime_2p_plus_1(size: usize) -> FF {
    let seed = generate_random_seed();
    let mut rng = Random::new(seed);
    let (mut is_prime, mut iteration) = (false, 0);
    let mut prime = FF::new(BIG_SIZE);

    while !is_prime {
        iteration += 1;
        prime = generate_prime(size);
        let mut prime_for_check = FF::mul(&prime, &FF::from_hex("2", BIG_SIZE));
        prime_for_check.inc(1);
        is_prime = FF::is_prime(&prime_for_check, &mut rng);
        debug!("Iteration: {}\nFound prime: {}\nis_prime: {}\n", iteration, prime, is_prime);
    }
    prime
}

pub fn random_qr(n: &FF){
    println!("n :{}", n);
    let mut random = random_in_range(&FF::from_hex("0", BIG_SIZE), n);
    println!("random :{}", random);
    random = FF::sqr(&random);
    println!("random sqr :{}", random);
//    let mut nn = n.clone();
//    nn.set_size(32);
//    random.set_size(32);
    let random1 = FF::modulus(&random, &n);
    println!("random1 :{}", random1);
//    let (mut ctx, mut random_qr) = (BigNumContext::new().unwrap(), BigNum::new().unwrap());
//    random_qr.sqr(&AnoncredsService::random_in_range(&BigNum::from_u32(0).unwrap(), &n), &mut ctx);
//    random_qr
}

fn random_in_range(start: &FF, end: &FF) -> FF {
    let sub = end - start;
    let size = significant_bits(&sub);
    let mut random_number = generate_big_random(size);

    while (&random_number + start) > *end {
        random_number = generate_big_random(size);
    }

    random_number = &random_number + start;
    debug!("start: {}\nend: {}\nsub: {}\nrandom: {}", start, end, sub, random_number);
    random_number
}

pub fn encode_attribute(attribute: &str, byte_order: ByteOrder) {
    let array_bytes = attribute.as_bytes();
    let mut sha256: hash256 = hash256::new();
    println!("array_bytes{:?}", array_bytes);
    for byte in array_bytes[..].iter() {
        sha256.process(*byte);
    }

    let mut hashed_array: Vec<u8> =
        sha256.hash().iter()
            .map(|v| *v as u8)
            .collect();

    if let ByteOrder::Little = byte_order {
        hashed_array.reverse();
    }
    println!("hashedarr{:?}", hashed_array);
    let hex = FF::from_bytes(&hashed_array[..], hashed_array.len(), BIG_SIZE);
    println!("asby{}", hex);
    //    let mut result = hash(MessageDigest::sha256(), attribute.as_bytes()).unwrap();
    //    let index = result.iter().position(|&value| value == 0);
    //    if let Some(position) = index {
    //        result.truncate(position);
    //    }
    //    if let ByteOrder::Little = byte_order {
    //        result.reverse();
    //    }
    //    let encoded_attribute = AnoncredsService::transform_byte_array_to_big_integer(&result);
    //    encoded_attribute.to_dec_str().unwrap().to_string()
}

fn significant_bytes(n: &FF) -> Vec<u8> {
    let mut bytes = n.to_bytes();
    let length = bytes.len();
    let index = bytes.iter().position(|&value| value != 0);
    if let Some(index) = index {
        bytes.reverse();
        bytes.truncate(length - index);
        bytes.reverse();
    }
    bytes
}

fn significant_bits(n: &FF) -> usize {
    let bytes = significant_bytes(n);
    let mut result = (bytes.len() - 1) * 8;
    result += format!("{:b}", bytes[0]).len();
    result
}

fn generate_probable_prime(size: usize) {
    let mut random_number = generate_big_random(size);
    let mut mods: Vec<FF> = Vec::new();
    for i in 1..NUM_PRIMES {
        debug!("{}", i);
        let bytes = random_number.to_bytes();
        let mut new_random = FF::from_bytes(&bytes, size, BIG_SIZE);
        let prime = FF::from_hex(&format!("{:x}", PRIMES[i])[..], BIG_SIZE);
        FF::modulus(&mut new_random, &prime);
        mods.push(new_random);
    }
    //TODO loop for mods check
}

pub fn get_hash_as_int(num: FF) -> FF {
    let array_bytes: Vec<u8> = num.to_bytes();

    let index = array_bytes.iter().position(|&value| value != 0).unwrap_or(array_bytes.len());

    let mut sha256: hash256 = hash256::new();

    for byte in array_bytes[index..].iter() {
        sha256.process(*byte);
    }

    let mut hashed_array: Vec<u8> =
        sha256.hash().iter()
            .map(|v| *v as u8)
            .collect();

    hashed_array.reverse();

    FF::from_bytes(&hashed_array[..], hashed_array.len(), 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_in_range_works() {
        ::env_logger::init().unwrap();

        let start = generate_big_random(100);
        let mut end = generate_big_random(120);

        while end < start {
            end = generate_big_random(30);
        }

        let random = random_in_range(&start, &end);
        assert!((random > start) && (random < end));
    }

    #[test]
    fn test_random_qr() {
//        let n = generate_big_random(10);
//        random_qr(&n);
    }

    #[test]
    fn encode_attribute_works() {
        let test_str_one = "Alexer5435";
        let test_str_two = "Alexer";
        let test_answer_one = "f54a";
        let test_answer_two = "cf76920dae32802476cc5e8d2518fd21c16b5f83e713a684db1aeb7089c67091";
        encode_attribute(test_str_one, ByteOrder::Big);
//        assert_eq!(test_answer_one, AnoncredsService::encode_attribute(test_str_one, ByteOrder::Big));
//        assert_eq!(test_answer_two, AnoncredsService::encode_attribute(test_str_two, ByteOrder::Big));
    }
}