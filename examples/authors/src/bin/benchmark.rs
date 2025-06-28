// Performance benchmark for zero-cost abstraction verification

use authors::queries::*;

fn main() {
    println!("🔬 Zero-Cost Abstraction Benchmark");
    println!("=====================================");

    // Test functions
    #[inline(never)]
    fn create_with_type_state_pattern() -> GetAuthorByIdAndAge {
        GetAuthorByIdAndAge::builder().id(123).age(Some(25)).build()
    }

    #[inline(never)]
    fn create_direct() -> GetAuthorByIdAndAge {
        GetAuthorByIdAndAge {
            id: 123,
            age: Some(25),
        }
    }

    #[inline(never)]
    fn create_3_param_type_state() -> UpdateAuthorStatus {
        UpdateAuthorStatus::builder()
            .is_active(Some(true))
            .age(Some(30))
            .id(456)
            .build()
    }

    #[inline(never)]
    fn create_3_param_direct() -> UpdateAuthorStatus {
        UpdateAuthorStatus {
            is_active: Some(true),
            age: Some(30),
            id: 456,
        }
    }

    const ITERATIONS: usize = 10_000_000;

    println!("📊 Running {} iterations per test...", ITERATIONS);

    // ウォームアップ
    for _ in 0..1000 {
        std::hint::black_box(create_with_type_state_pattern());
        std::hint::black_box(create_direct());
        std::hint::black_box(create_3_param_type_state());
        std::hint::black_box(create_3_param_direct());
    }

    // 2パラメータ型状態パターン測定
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(create_with_type_state_pattern());
    }
    let type_state_duration = start.elapsed();

    // 2パラメータ直接構築測定
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(create_direct());
    }
    let direct_duration = start.elapsed();

    // 3パラメータ型状態パターン測定
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(create_3_param_type_state());
    }
    let type_state_3_duration = start.elapsed();

    // 3パラメータ直接構築測定
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(create_3_param_direct());
    }
    let direct_3_duration = start.elapsed();

    println!("\n📈 Results:");
    println!("-------------------");

    println!("🔹 2-Parameter Query (GetAuthorByIdAndAge):");
    println!(
        "   Type-state pattern: {:>8.2?} ({:>6.2} ns/op)",
        type_state_duration,
        type_state_duration.as_nanos() as f64 / ITERATIONS as f64
    );
    println!(
        "   Direct construction: {:>7.2?} ({:>6.2} ns/op)",
        direct_duration,
        direct_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    let overhead_2 = if direct_duration.as_nanos() > 0 {
        (type_state_duration.as_nanos() as f64 / direct_duration.as_nanos() as f64 - 1.0) * 100.0
    } else {
        0.0
    };
    println!("   Overhead: {:>13.2}%", overhead_2);

    println!("\n🔹 3-Parameter Query (UpdateAuthorStatus):");
    println!(
        "   Type-state pattern: {:>8.2?} ({:>6.2} ns/op)",
        type_state_3_duration,
        type_state_3_duration.as_nanos() as f64 / ITERATIONS as f64
    );
    println!(
        "   Direct construction: {:>7.2?} ({:>6.2} ns/op)",
        direct_3_duration,
        direct_3_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    let overhead_3 = if direct_3_duration.as_nanos() > 0 {
        (type_state_3_duration.as_nanos() as f64 / direct_3_duration.as_nanos() as f64 - 1.0)
            * 100.0
    } else {
        0.0
    };
    println!("   Overhead: {:>13.2}%", overhead_3);

    println!("\n📏 Memory Layout:");
    println!(
        "   GetAuthorByIdAndAge: {} bytes",
        std::mem::size_of::<GetAuthorByIdAndAge>()
    );
    println!(
        "   UpdateAuthorStatus:  {} bytes",
        std::mem::size_of::<UpdateAuthorStatus>()
    );

    println!("\n🏆 Final Assessment:");

    if overhead_2.abs() <= 5.0 && overhead_3.abs() <= 5.0 {
        println!("   ✅ ZERO-COST ABSTRACTION ACHIEVED!");
        println!("   ✅ Type-state pattern performance is equivalent to direct construction");
        println!("   ✅ Compile-time safety with runtime efficiency");
    } else if overhead_2 <= 20.0 && overhead_3 <= 20.0 {
        println!(
            "   ⚡ Very low overhead detected ({:.1}%, {:.1}%)",
            overhead_2, overhead_3
        );
        println!("   ✅ Excellent performance with compile-time safety benefits");
    } else {
        println!("   ⚠️  Significant overhead detected - optimization needed");
        println!(
            "   📊 2-param: {:.1}%, 3-param: {:.1}%",
            overhead_2, overhead_3
        );
    }
}
