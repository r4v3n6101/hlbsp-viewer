#[test]
fn print_entries_name() {
    let file = std::fs::read(env!("WAD_TEST")).unwrap();
    let wad = wad::Archive::parse(&file).unwrap();
    wad.entries().for_each(|(&name, _)| println!("{}", name));
}
