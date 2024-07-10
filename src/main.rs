mod digest;
// use digest::Digest;

// CAS = Content-addressed store
// -----------------------------
//
// Files are named according to the the hash of the contents. Files with equal
// contents will have the same name and naturally be deduplicated (only stored
// once). Even a slight difference in content will make the name different.
//
//
// Label Storage = Mutable
// -----------------------
//
// Labels map a human-readable name to some hash that can be found in the CAS.
// Labels can change over time, allowing a new version of a file or directory
// to become canon.

fn main() {
    println!("Hello, world!");
}
