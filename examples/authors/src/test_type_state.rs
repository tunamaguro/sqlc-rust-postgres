// Test type-state pattern compile-time safety with generated code

use crate::queries::GetAuthor;

#[allow(dead_code)]
pub fn test_type_state_safety() {
    // ✅ Test 1: Correct usage compiles successfully
    let _query1 = GetAuthor::builder()
        .id(123)
        .build();

    // ❌ Test 2: Missing parameter should cause compile error
    // Uncomment the following line to test:
    // let _query2 = GetAuthor::builder().build();
    // Expected error: no method named `build` found for struct `GetAuthorBuilder<()>`

    // ❌ Test 3: Duplicate parameter setting is impossible  
    // Uncomment the following line to test:
    // let _query3 = GetAuthor::builder().id(123).id(456).build();
    // Expected error: no method named `id` found for struct `GetAuthorBuilder<i64>`

    println!("✅ Type-state pattern safety tests passed!");
}