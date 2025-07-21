use crate::core::models::{DataPath, Employee};
use std::fs;
use std::io::{self, Write};

pub struct EmployeeService;

impl EmployeeService {
    pub fn add_employee(data_path: &DataPath, employee_name: &str) -> io::Result<()> {
        println!("Adding new employee: {employee_name}");
        print!("Title: ");
        io::stdout().flush()?;
        let mut title = String::new();
        io::stdin().read_line(&mut title)?;

        let employee = Employee {
            name: employee_name.to_string(),
            title: title.trim().to_string(),
        };

        let toml = toml::to_string(&employee).unwrap();
        let path = data_path
            .employees_dir
            .join(format!("{employee_name}.toml"));
        fs::write(path, toml)?;
        println!("Employee '{employee_name}' added.");
        Ok(())
    }

    pub fn employee_exists(data_path: &DataPath, employee_name: &str) -> bool {
        let employee_file = data_path
            .employees_dir
            .join(format!("{employee_name}.toml"));
        employee_file.exists()
    }

    pub fn list_employees(data_path: &DataPath) -> io::Result<Vec<String>> {
        let mut employees = Vec::new();

        if !data_path.employees_dir.exists() {
            return Ok(employees);
        }

        for entry in fs::read_dir(&data_path.employees_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    employees.push(name.to_string());
                }
            }
        }

        employees.sort();
        Ok(employees)
    }

    #[allow(dead_code)]
    pub fn get_employee(data_path: &DataPath, employee_name: &str) -> io::Result<Employee> {
        let employee_file = data_path
            .employees_dir
            .join(format!("{employee_name}.toml"));
        let content = fs::read_to_string(employee_file)?;
        let employee: Employee =
            toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(employee)
    }
}
