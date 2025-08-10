use kzen_paillier::{Decrypt, EncodedCiphertext, Encrypt, KeyGeneration, Keypair, Paillier, RawCiphertext};
use rand::{thread_rng};
mod rng_test;
use curv::arithmetic::traits::{ Converter, Modulo};
use curv::arithmetic::BigInt;
use rand::prelude::*;



#[derive(serde::Deserialize)]
pub struct EncodedCiphertextRepr<T> {
    raw: String,
    components: usize,
    _phantom: std::marker::PhantomData<T>,
}


fn key_setup(n: usize) -> Vec<(Keypair)> {
    let mut keys: Vec<Keypair>=vec![];
    for _ in 0..n {
        let keyp:Keypair = Paillier::keypair();
        keys.push(keyp);
    }
    keys

}

// fn square_and_multiply(keys: &Keypair, base: &EncodedCiphertext<u64>, exponent: &u64) -> EncodedCiphertext<u64> {
//     let mut result = Paillier::encrypt(&keys.keys().0, 1u64);
//     let mut current_base = base.clone();
//     let mut current_exponent = exponent.clone();

//     while current_exponent != 0u64 {
//         // let serialized_base = BigInt::modpow;
//         if current_exponent & 1u64 != 0u64 {
//             result = Paillier::mul(&keys.keys().0, &result, plaintext);
//         }
//         //serialize the base and multiply it with itself
//         current_base = Paillier::mul(&keys.keys().0, &current_base, plaintext);
//         current_exponent = current_exponent >> 1; // Divide by 2
//     }
//     result
// }

fn get_bigint_from_encoded_ciphertext<T>(encoded: &EncodedCiphertext<T>) -> BigInt {
    let encrypted_string = serde_json::to_string(&encoded).unwrap();
    let deserialized: EncodedCiphertextRepr<u64> = serde_json::from_str(&encrypted_string).unwrap();
    return BigInt::from_str_radix(deserialized.raw.as_str(), 10).expect("Failed to convert encoded ciphertext to BigInt")
}

fn main() {

    // key_setup for 5 users;
    let keys = key_setup(5);
    let secret_inputs = [1,2,3,4,1000];
    //first user encrypts its secret input
    let encrypted_input = Paillier::encrypt(&keys[0].keys().0, secret_inputs[0]);
    //extract n^2 from the paillier setup
    let n_squared = keys[0].keys().0.nn;
    let n = keys[0].keys().0.n;
    // println!("Encrypted input: {:?}", encrypted_input);

    let last_encrypted_value = encrypted_input;
    let mut last_encrypted_value: BigInt =  get_bigint_from_encoded_ciphertext(&last_encrypted_value);
    // println!("Last encrypted value: {:?}", last_encrypted_value);
    println!("Secret input for user {}: {:?}", 0, secret_inputs[0]);

    let mut encrypted_values: Vec<BigInt> = vec![];
    encrypted_values.push(last_encrypted_value.clone());
    for (i,_) in secret_inputs.iter().enumerate().skip(1) {
        //get the encrypted input from the previous user and encrypt it
        println!("Secret input for user {}: {:?}", i, secret_inputs[i]);
        // let raised_value = base.pow(secret_inputs[i]);x
        let a = &last_encrypted_value;
        let b = &BigInt::from(secret_inputs[i]);
        let c = &n_squared;
        let new_value = BigInt::mod_pow(a, b,c );
        last_encrypted_value = new_value;
        // println!("Encrypted value for user {}: {:?}", i, last_encrypted_value);
        encrypted_values.push(last_encrypted_value.clone());
        // encrypted_values.push(&last_encrypted_value);
        // println!("Encrypted value for user {}: {:?}", i, encrypted_values.last().unwrap());
    }
    println!("--------------------");
    //now the process of encrypting the random values for each user from the other side (back to the first user)
    encrypted_values.reverse();
    let nums: Vec<i32> = (1..100).collect();
    let mut random_values: Vec<BigInt> = vec![];
    for (i,_) in encrypted_values.iter().skip(1).enumerate() {
        //get the encrypted input from the previous user and encrypt it
        let rng:u64 = nums.choose(&mut thread_rng()).unwrap().clone() as u64;
        random_values.push(BigInt::from(rng));
        let encrypted_random_value = Paillier::encrypt(&keys[0].keys().0, rng);
        //get the bigint out of the encodedCiphertext
        let encrypted_value = get_bigint_from_encoded_ciphertext(&encrypted_random_value);
        //find the inverse of the encrypted value
        let inverse_encrypted_value = BigInt::mod_inv(&encrypted_value, &n_squared).unwrap();
        // println!("Inverse encrypted value for user {}: {:?}", i, inverse_encrypted_value);
        //multiply the last encrypted value with the inverse of the encrypted value
        last_encrypted_value = BigInt::mod_mul(&last_encrypted_value, &inverse_encrypted_value, &n_squared);
        // println!("Last encrypted value for user {}: {:?}", i, last_encrypted_value);
    }
    let last_encrypted_value = RawCiphertext::from(last_encrypted_value);
    // now the last decrypted value should be equal to the first user's secret input
    let decrypted_value = Paillier::decrypt(&keys[0].keys().1, &last_encrypted_value);
    // println!("Decrypted value: {:?}", decrypted_value);
    random_values.push(decrypted_value.0.into_owned());

    let mut multiplication_result: BigInt = BigInt::from(0);
    for (i,_) in random_values.iter().enumerate(){
        print!("Random share for user {}: {:?}\n", random_values.len()-1-i, random_values[i]);
        multiplication_result = BigInt::mod_add(&random_values[i], &multiplication_result, &n);

    }
    println!("Addition of the random shares : {:?}", multiplication_result);
    println!("Multiplication of the secret inputs: {:?}", secret_inputs.iter().fold(BigInt::from(1), |acc, &x| acc * BigInt::from(x)));
        
}

mod tests{
    use super::*;
    #[test]
    fn test_gen_keypair() {
        let keys = key_setup(5);
        assert_eq!(keys.len(), 5);
        for key in keys {
            println!("Keypair: {:?} {:?}", key.keys().0, key.keys().1);
        }
    }
}
