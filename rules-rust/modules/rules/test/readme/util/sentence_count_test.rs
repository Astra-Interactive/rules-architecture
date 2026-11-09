use crate::readme::util::count_sentences;

#[test]
fn given_only_a_title_when_counted_then_returns_zero() {
    assert_eq!(0, count_sentences("# Billing\n\n"));
}

#[test]
fn given_abbreviations_and_a_missing_stop_when_counted_then_returns_one() {
    assert_eq!(
        1,
        count_sentences("Wraps the U.S. Postal API, i.e. its tracking calls, etc. vs. a stub")
    );
}

#[test]
fn given_two_sentences_over_lines_when_counted_then_returns_two() {
    assert_eq!(2, count_sentences("Parses invoices.\nPrints them!\n"));
}

#[test]
fn given_an_abbreviation_inside_a_word_when_counted_then_keeps_its_stops() {
    assert_eq!(3, count_sentences("Uses the vse.g. format. Then ends."));
}
