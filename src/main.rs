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
      Adding sort operation, this is the contract GET * X SORT_BY Y
    */

    fn query(&self, command: &str) -> Option<Vec<String>> {
        let mut command_sections: VecDeque<&str> = command.split(' ').collect();
        let operation = Operation::from(command_sections.pop_front().unwrap());
        let sort_column = command_sections.pop_back().unwrap();
        let predicate = command_sections.pop_back().unwrap(); 
        let table_name = command_sections.pop_back().unwrap();
        let column_specifier = command_sections.pop_front().unwrap();
        println!("Operation: {:?}", operation);
        println!("Table: {}", table_name);
        println!("Spec: {}", column_specifier);
        // This works, but always assumes a fixed command, needs to work even if sort or other
        // commands are not present, also needs to consider other operations, or maybe move the
        // rest of the commands after getting the operation?

        println!("predicate {}, sort_column {}", predicate, sort_column);

        match operation {
            Operation::GET => {
                let query_table = self.get_table(table_name).unwrap();
                match column_specifier {
                    "*" => {
                        let all_data = query_table.data.read().unwrap();
                        all_data.as_ref().cloned()
                    }
                    _ => {
                        let column_coll: Vec<&str> = column_specifier.split(',').collect();
                        let column_indexes: Vec<u8> = column_coll
                            .into_iter()
                            .map(|col_name| query_table.columns.get(col_name).copied().unwrap())
                            .collect();

                        let mut result: Vec<String> = Vec::new();
                        let all_data = query_table.data.read().unwrap();

                        if let Some(rows) = all_data.as_ref() {
                            for row in rows {
                                let row_as_vec: Vec<&str> = row.split(',').collect();
                                let selected = column_indexes
                                    .iter()
                                    .map(|&index| row_as_vec[index as usize])
                                    .collect::<Vec<&str>>()
                                    .join(",");
                                result.push(selected);
                            }
                        }
                        if predicate == "SORT_BY" {
                            let cloned_query_table = Arc::clone(&query_table);
                            result.sort_by(|a,b| {
                                let split_a: Vec<&str> = a.split(',').collect();
                                let split_b: Vec<&str> = b.split(',').collect();
                                let sort_index = cloned_query_table.columns.get(sort_column).copied().unwrap();
                                println!("sorting value {}", split_a[sort_index as usize]);
                                split_a[sort_index as usize].cmp(split_b[sort_index as usize])
                            })
                        }
                        Some(result)
                    }
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
    table_append(&mut table1, "A,Z,70,M".to_string());
    let mut table_hashmap = HashMap::new();
    table_hashmap.insert("personas".to_string(), Arc::new(table1));
    let db = DB {
        id: 0,
        name: "New DB".to_string(),
        tables: Arc::new(RwLock::new(table_hashmap)),
    };

    if let Some(result) = db.query("GET name,sex,last_name personas SORT_BY age") {
        println!("result is = {:?}", result);
    }
}
