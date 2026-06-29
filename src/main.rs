pub trait Greeter {
    fn greet(&self) -> String;
    fn formal_greeting(&self, title: &str) -> String;
}

pub struct Person {
    pub name: String,
    age: u32,
}

impl Person {
    pub fn new(name: &str, age: u32) -> Self {
        Person {
            name: name.to_string(),
            age,
        }
    }

    pub fn get_age(&self) -> u32 {
        self.age
    }

    pub fn birthday(&mut self) {
        self.age += 1;
    }
}

impl Greeter for Person {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name)
    }

    fn formal_greeting(&self, title: &str) -> String {
        format!("Greetings, {} {}.", title, self.name)
    }
}

pub struct Team {
    pub name: String,
    members: Vec<Person>,
}

impl Team {
    pub fn new(name: &str) -> Self {
        Team {
            name: name.to_string(),
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, person: Person) {
        self.members.push(person);
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

pub fn create_default_team() -> Team {
    let mut team = Team::new("Default");
    team.add_member(Person::new("Alice", 30));
    team.add_member(Person::new("Bob", 25));
    team
}

fn main() {
    let person = Person::new("Charlie", 42);
    println!("{}", person.greet());
    println!("Age: {}", person.get_age());

    let team = create_default_team();
    println!("Team {} has {} members", team.name, team.member_count());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_creation() {
        let p = Person::new("Test", 25);
        assert_eq!(p.get_age(), 25);
    }

    #[test]
    fn test_greeting() {
        let p = Person::new("World", 0);
        assert!(p.greet().contains("World"));
    }

    #[test]
    fn test_team_creation() {
        let team = create_default_team();
        assert_eq!(team.member_count(), 2);
    }

    #[test]
    fn test_greeter_trait() {
        let p = Person::new("Dave", 30);
        assert_eq!(p.formal_greeting("Mr"), "Greetings, Mr Dave.");
    }
}
