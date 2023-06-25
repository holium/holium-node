use crate::get_connection;
use crate::models::Passport;
use diesel::prelude::*;

pub fn db_get_contact_passports(phone_numbers: Vec<String>) {
    use crate::schema::passports::dsl::*;
    let mut conn = get_connection().expect("Failed to get database connection");

    let results = passports
        .filter(is_public.eq(true))
        .filter(phone_number.eq_any(phone_numbers))
        .load::<Passport>(&mut conn)
        .expect("Error loading passports");

    println!("Displaying {} passports", results.len());
    for passport in results {
        println!("{:?}", passport.phone_number);
        println!("-----------\n");
        println!("{}", passport.ship);
    }
}

pub fn db_insert_passport(passport: Passport) -> Passport {
    use crate::schema::passports::dsl::*;
    let mut conn = get_connection().expect("Failed to get database connection");

    diesel::insert_into(passports)
        .values(&passport)
        .on_conflict(ship)
        .do_update()
        .set((
            is_public.eq(&passport.is_public),
            nickname.eq(&passport.nickname),
            color.eq(&passport.color),
            twitter.eq(&passport.twitter),
            bio.eq(&passport.bio),
            avatar.eq(&passport.avatar),
            cover.eq(&passport.cover),
            featured_url.eq(&passport.featured_url),
            phone_number.eq(&passport.phone_number),
            updated_at.eq(&chrono::Local::now().naive_local()),
        ))
        .get_result::<Passport>(&mut conn)
        .expect("Error saving new passport")
}
