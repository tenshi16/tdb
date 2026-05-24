use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Debug)]
enum Operation {
    GET,
    DELETE,
    UPDATE,
    INSERT,
}

impl From<&str> for Operation {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Operation::GET,
            "DELETE" => Operation::DELETE,
            "UPDATE" => Operation::UPDATE,
            "INSERT" => Operation::INSERT,
            _ => Operation::GET,
        }
    }
}

struct Table {
    id: u64,
    name: String,
    row_count: u64,
    columns: HashMap<String, u8>,
    data: Arc<RwLock<Option<Vec<String>>>>,
}

struct DB {
    id: u64,
    name: String,
    tables: Arc<RwLock<HashMap<String, Arc<Table>>>>,
}

impl DB {
    fn new(name: &str) -> Self {
        DB {
            id: 0,
            name: name.to_string(),
            tables: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    fn create_table(&self, table: Table) {
        let mut tables = self.tables.write().unwrap();
        tables.entry(table.name.clone()).or_insert(Arc::new(table));
    }
    fn get_table(&self, name: &str) -> Option<Arc<Table>> {
        let table = self.tables.read().unwrap();
        table.get(name).cloned()
    }

    /*
      Let's start with a simple command, this is the contract GET * FROM X
    */

    fn query(&self, command: &str) -> Option<Vec<String>> {
        let mut command_sections: VecDeque<&str> = command.split(' ').collect();
        let operation: Operation;
        let table_name: &str;
        let column_specifier: &str;

        operation = Operation::from(command_sections.pop_front().unwrap());
        table_name = command_sections.pop_back().unwrap();
        column_specifier = command_sections.pop_front().unwrap();
        println!("op{:?}", operation);
        println!("table{}", table_name);
        println!("spec{}", column_specifier);

        match operation {
            Operation::GET => {
                let query_table = self.get_table(table_name).unwrap();
                match column_specifier {
                    "*" => {
                        let all_data = query_table.data.read().unwrap();
                        all_data.as_ref().cloned()
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

fn table_append(table: &mut Table, data: String) {
    let clone_data = Arc::clone(&table.data);
    thread::spawn(move || {
        let mut inner_data = clone_data.write().unwrap();
        if let Some(vec) = inner_data.as_mut() {
            vec.push(data);
        }
    })
    .join()
    .unwrap();
    table.row_count += 1;
}

fn table_print(table: &Table) {
    let clone_data = Arc::clone(&table.data);
    thread::spawn(move || {
        if let Some(data) = clone_data.read().unwrap().as_ref() {
            println!("{:?}", data);
        }
    })
    .join()
    .unwrap();
}

fn main() {
    let data = Arc::new(RwLock::new(Some(vec![])));
    let mut columns = HashMap::new();
    columns.insert("name".to_string(), 0);
    columns.insert("last_name".to_string(), 1);
    columns.insert("age".to_string(), 2);
    columns.insert("sex".to_string(), 3);
    let mut table1 = Table {
        id: 1,
        name: "Test table".to_string(),
        row_count: 0,
        columns: columns,
        data: data,
    };

    table_append(&mut table1, "Angel,Gomez,30,M".to_string());
    table_append(&mut table1, "M,G,29,F".to_string());
    let mut table_hashmap = HashMap::new();
    table_hashmap.insert("personas".to_string(), Arc::new(table1));
    let db = DB {
        id: 0,
        name: "New DB".to_string(),
        tables: Arc::new(RwLock::new(table_hashmap)),
    };

    if let Some(result) = db.query("GET * personas") {
        println!("result is = {:?}", result);
    }
}
