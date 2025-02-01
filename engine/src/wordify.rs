pub fn wordify_airline<'a>(airline: String) -> &'a str {
  match airline.as_str() {
    "AAL" => "American",
    "JBU" => "JetBlue",
    "SKW" => "SkyWest",
    _ => "Unknown",
  }
}

fn wordify_digit<'a>(digit: char) -> &'a str {
  match digit {
    '0' => "zero",
    '1' => "one",
    '2' => "two",
    '3' => "three",
    '4' => "four",
    '5' => "five",
    '6' => "six",
    '7' => "seven",
    '8' => "eight",
    '9' => "nine",
    _ => "unknown",
  }
}

fn wordify_digit_lsb(digit: char) -> String {
  if digit == '0' {
    "".to_owned()
  } else {
    format!("-{}", wordify_digit(digit))
  }
}

fn wordify_pair(pair: (char, char)) -> String {
  match pair {
    ('0', '0') => "zero zero".to_owned(),
    ('0', '1') => "zero one".to_owned(),
    ('0', '2') => "zero two".to_owned(),
    ('0', '3') => "zero three".to_owned(),
    ('0', '4') => "zero four".to_owned(),
    ('0', '5') => "zero five".to_owned(),
    ('0', '6') => "zero six".to_owned(),
    ('0', '7') => "zero seven".to_owned(),
    ('0', '8') => "zero eight".to_owned(),
    ('0', '9') => "zero nine".to_owned(),
    ('1', '0') => "ten".to_owned(),
    ('1', '1') => "eleven".to_owned(),
    ('1', '2') => "twelve".to_owned(),
    ('1', '3') => "thirteen".to_owned(),
    ('1', '4') => "fourteen".to_owned(),
    ('1', '5') => "fifteen".to_owned(),
    ('1', '6') => "sixteen".to_owned(),
    ('1', '7') => "seventeen".to_owned(),
    ('1', '8') => "eighteen".to_owned(),
    ('1', '9') => "nineteen".to_owned(),
    ('2', n) => format!("twenty{}", wordify_digit_lsb(n)),
    ('3', n) => format!("thirty{}", wordify_digit_lsb(n)),
    ('4', n) => format!("forty{}", wordify_digit_lsb(n)),
    ('5', n) => format!("fifty{}", wordify_digit_lsb(n)),
    ('6', n) => format!("sixty{}", wordify_digit_lsb(n)),
    ('7', n) => format!("seventy{}", wordify_digit_lsb(n)),
    ('8', n) => format!("eighty{}", wordify_digit_lsb(n)),
    ('9', n) => format!("ninety{}", wordify_digit_lsb(n)),
    _ => "unknown".to_owned(),
  }
}

pub fn wordify_flight_number(flight_number: String) -> String {
  let chunks = flight_number.chars().collect::<Vec<char>>();
  let chunks = chunks.chunks(2).collect::<Vec<&[char]>>();

  let mut string = String::new();
  for chunk in chunks {
    string.push_str(&wordify_pair((chunk[0], chunk[1])));
    string.push(' ');
  }

  string.trim().to_owned()
}

pub fn wordify<T: AsRef<str>>(text: T) -> String {
  let text = text.as_ref();
  let airline = text[0..3].to_string();
  let flight_number = text.chars().skip(3).collect::<String>();

  format!(
    "{} {}",
    wordify_airline(airline),
    wordify_flight_number(flight_number)
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_wordify_airline() {
    assert_eq!(wordify_airline("AAL".to_owned()), "American");
    assert_eq!(wordify_airline("JBU".to_owned()), "JetBlue");
    assert_eq!(wordify_airline("SKW".to_owned()), "SkyWest");
    assert_eq!(wordify_airline("XYZ".to_owned()), "Unknown");
  }

  #[test]
  fn test_wordify_digit() {
    assert_eq!(wordify_digit('0'), "zero");
    assert_eq!(wordify_digit('1'), "one");
    assert_eq!(wordify_digit('2'), "two");
    assert_eq!(wordify_digit('3'), "three");
    assert_eq!(wordify_digit('4'), "four");
    assert_eq!(wordify_digit('5'), "five");
    assert_eq!(wordify_digit('6'), "six");
    assert_eq!(wordify_digit('7'), "seven");
    assert_eq!(wordify_digit('8'), "eight");
    assert_eq!(wordify_digit('9'), "nine");
    assert_eq!(wordify_digit('X'), "unknown");
  }

  #[test]
  fn wordify_1234() {
    assert_eq!(wordify_pair(('1', '2')), "twelve");
    assert_eq!(wordify_pair(('3', '4')), "thirty-four");
    assert_eq!(wordify("AAL1234"), "American twelve thirty-four");
  }

  #[test]
  fn wordify_0040() {
    assert_eq!(wordify("AAL0040"), "American zero zero forty");
  }

  #[test]
  fn wordify_0000() {
    assert_eq!(wordify("AAL0000"), "American zero zero zero zero");
  }
}
