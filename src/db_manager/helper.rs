use std::path::{Path, PathBuf};

use rusqlite::{Result, Rows};

use super::creation::DatabaseConnection;

// counts how many rows the sql select statement outputed
pub fn count_rows(rows: Result<Rows<'_>>) -> Result<usize, rusqlite::Error> {
    let values: Vec<_> = rows?.mapped(|_| Ok(())).collect();

    Ok(values.len())
}

pub fn gen_random_path() -> PathBuf {
    let path = format!("./{}.db3", uuid::Uuid::new_v4().to_string());

    Path::new(&path).to_owned()
}

pub fn gen_vals(conn: &DatabaseConnection) {
    let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to, in_use, visited, content) VALUES (?1, ?2, ?3, ?4, ?5);") 
            .unwrap();

    prep.execute((
        "test.ch",
        "team-crystal.ch:::google.ch:::example.com",
        false,
        true,
        "team-crystal.ch:::google.ch:::example.com",
    ))
    .unwrap();
    prep.execute((
        "help.ch",
        "team-crystal.ch:::google.ch:::test.ch",
        false,
        true,
        "team-crystal.ch:::google.ch:::test.ch",
    ))
    .unwrap();
    prep.execute(("p.ch", "help.ch", false, true, "help.ch"))
        .unwrap();
    prep.execute((
        "ep.ch",
        "team-crystal.ch:::help.ch",
        false,
        true,
        "team-crystal.ch::help.ch",
    ))
    .unwrap();
    prep.execute((
        "lp.ch",
        "help.ch:::google.ch",
        false,
        true,
        "help.ch:::google.ch",
    ))
    .unwrap();

    prep.execute((
            "hre.he",
            "test.ch:::lp.ch",
            false,
            true,
            "<html><body><h1>
            Laborum nulla quis deserunt labore quis cupidatat reprehenderit amet consequat reprehenderit tempor anim sint amet. Eiusmod fugiat eu aliqua qui do proident adipisicing. Dolore esse laborum voluptate in qui in ex. Sunt exercitation sit dolore cillum. Nostrud non aliqua sit anim aliqua labore Lorem quis nostrud. Exercitation ex nulla in laborum eu non voluptate consectetur.
            Incididunt anim voluptate aliqua et commodo cillum. Adipisicing fugiat ea consectetur cupidatat quis velit duis. Ad fugiat id quis proident qui mollit eu fugiat exercitation. Consectetur velit tempor esse reprehenderit laboris ea labore consectetur ut irure cupidatat in mollit. Dolore consequat amet id ipsum deserunt in eiusmod. Sunt excepteur eu eiusmod voluptate est mollit elit sunt laboris nostrud. Culpa non ea ad ex veniam et aute.

            Tempor enim non laborum enim ut duis laborum. Dolore nisi dolor Lorem anim occaecat non eu tempor incididunt. Consectetur aliquip reprehenderit fugiat magna. Est voluptate nisi id voluptate est cupidatat incididunt. Aute est qui mollit quis commodo irure ut eu ipsum sit ex cupidatat est adipisicing. Amet qui do cillum duis ad. Voluptate anim ipsum mollit sint incididunt.

            Eu nisi eu quis anim tempor fugiat deserunt est deserunt nulla ad do. Ipsum pariatur enim eiusmod minim cupidatat esse excepteur nostrud proident officia Lorem laboris esse. Excepteur reprehenderit anim duis exercitation labore nisi aliquip duis do. Id eiusmod dolore ex nulla nulla.
            </h1></body></html>",
        ))
        .unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn correct_count() {
        todo!()
        // Test if count_rows outputs correct value
    }

    #[test]
    fn is_random() {
        todo!()
        // test if gen_random_path outputs a random path (generate 10000 paths and everyone has to be unique)
    }

    #[test]
    fn is_path() {
        todo!()
        // test if gen_random_path generates a correct path
    }

    #[test]
    fn vals_generated() {
        todo!()
        // test if gen_vals generates the correct values
    }
}
