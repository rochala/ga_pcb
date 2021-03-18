use super::*;

fn setup() -> Individual {
    let mut pin_locations = vec![];
    pin_locations.push(((1, 3), (5, 3)));
    generate_individual((6, 6), pin_locations, Some(1))
}

#[test]
fn test_mutation() {
    let mut individual = setup();

    println!("{}", individual);

    individual.connections[0].mutate_segment((0.3, 0.7), (6, 6));
    println!("{}", individual);

    for i in 0..100000 {
        let mut pin_locations = vec![];
        pin_locations.push(((1, 3), (5, 3)));
        individual = generate_individual((6, 6), pin_locations, Some(i));
        println!("{}", individual);
        individual.connections[0].mutate_segment((0.3, 0.7), (6, 6));
        println!("{}", individual);
        println!("------------------------------");
    }

//     individual.connections[0] = Connection{
//         start: (2,2),
//         end: (5,2),
//         segments: vec![
//             Segment{length: 1, direction: South},
//             Segment{length: 1, direction: East},
//             Segment{length: 1, direction: South},
//             Segment{length: 1, direction: West},
//             Segment{length: 1, direction: South},
//         ],
//     };
//     println!("{}", individual);
//     individual.connections[0].mutate_segment((0.23, 0.2), (6, 6));
//     println!("{:?}", individual.connections[0]);
//     println!("{}", individual);
}

#[test]
fn test_find_point() {
    let individual = setup();
    assert_eq!(individual.connections[0].find_point(0), (1, 0));
    assert_eq!(individual.connections[0].find_point(1), (5, 0));
    assert_eq!(individual.connections[0].find_point(2), (5, 3));
}
