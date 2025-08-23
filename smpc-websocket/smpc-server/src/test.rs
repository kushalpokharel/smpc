use curv::arithmetic::{BigInt,Converter};

mod tests{
    use curv::arithmetic::Modulo;

    use super::*;
    #[test]
    fn test_outputs() {
        let a = BigInt::from_str_radix("363", 10).unwrap();
        let b = BigInt::from(7u64);
        let c = BigInt::from_str_radix("ccdba028de01767d1946de20209389cf00f338346ffd4a49f4922f57aa788a4790de3c9915595fd8bfd6105348ceb09f6e19774d2b8e5efd832839633671c2377d15f874a8f9c9b86cccf834dbcee63093cb92cb1a21d535d748d5d61d9913bd192b3cc780f90df41e8d9dff9b05150309b44bb845b13bbeb86a945d79e10c20c158ee810fdd4d73a4c6cab047b8883898aff807d897d455d2a02214c817d53ca8429763d4847a110190e9327d75584c1a61a4825524bc952578b35f92b66a62cf60ef255019629ed64d0f752f14054aecfa14d5e56038db0ddb76565764149ffeb2e63d1e271f9024be1b60ae97dc491a5f053aa06e3bc8324f763ad7d04e27", 16).unwrap();
        let result = BigInt::mod_add(&a, &b, &c);
        assert!(a<c, "Decrypted value should be less than n");
        let mul = BigInt::mod_mul(&BigInt::from(10),&BigInt::from(37), &c);
        assert!(result==mul);
    }
}
