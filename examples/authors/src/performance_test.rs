// Performance test for zero-cost abstraction verification

use crate::queries::*;

#[allow(dead_code)]
pub fn benchmark_type_state_pattern() {
    // ベンチマーク関数: 型状態パターン vs 直接構築の比較

    // Test 1: 型状態パターンでの構築
    #[inline(never)]
    fn create_with_type_state_pattern() -> GetAuthorByIdAndAge {
        GetAuthorByIdAndAge::builder().id(123).age(Some(25)).build()
    }

    // Test 2: 直接構築（理想的なゼロコスト）
    #[inline(never)]
    fn create_direct() -> GetAuthorByIdAndAge {
        GetAuthorByIdAndAge {
            id: 123,
            age: Some(25),
        }
    }

    // Test 3: 3パラメータの型状態パターン
    #[inline(never)]
    fn create_3_param_type_state() -> UpdateAuthorStatus {
        UpdateAuthorStatus::builder()
            .is_active(Some(true))
            .age(Some(30))
            .id(456)
            .build()
    }

    // Test 4: 3パラメータの直接構築
    #[inline(never)]
    fn create_3_param_direct() -> UpdateAuthorStatus {
        UpdateAuthorStatus {
            is_active: Some(true),
            age: Some(30),
            id: 456,
        }
    }

    // パフォーマンステスト実行
    let iterations = 1_000_000;

    // ウォームアップ
    for _ in 0..1000 {
        let _ = create_with_type_state_pattern();
        let _ = create_direct();
        let _ = create_3_param_type_state();
        let _ = create_3_param_direct();
    }

    // 型状態パターン測定
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = create_with_type_state_pattern();
    }
    let type_state_duration = start.elapsed();

    // 直接構築測定
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = create_direct();
    }
    let direct_duration = start.elapsed();

    // 3パラメータ型状態パターン測定
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = create_3_param_type_state();
    }
    let type_state_3_duration = start.elapsed();

    // 3パラメータ直接構築測定
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = create_3_param_direct();
    }
    let direct_3_duration = start.elapsed();

    println!("🔬 Zero-Cost Abstraction Verification:");
    println!("📊 2-Parameter Query (GetAuthorByIdAndAge):");
    println!("   Type-state pattern: {:?}", type_state_duration);
    println!("   Direct construction: {:?}", direct_duration);
    println!(
        "   Overhead: {:.2}%",
        (type_state_duration.as_nanos() as f64 / direct_duration.as_nanos() as f64 - 1.0) * 100.0
    );

    println!("📊 3-Parameter Query (UpdateAuthorStatus):");
    println!("   Type-state pattern: {:?}", type_state_3_duration);
    println!("   Direct construction: {:?}", direct_3_duration);
    println!(
        "   Overhead: {:.2}%",
        (type_state_3_duration.as_nanos() as f64 / direct_3_duration.as_nanos() as f64 - 1.0)
            * 100.0
    );

    // 構造体サイズ確認
    println!("📏 Memory Layout:");
    println!(
        "   GetAuthorByIdAndAge size: {} bytes",
        std::mem::size_of::<GetAuthorByIdAndAge>()
    );
    println!(
        "   UpdateAuthorStatus size: {} bytes",
        std::mem::size_of::<UpdateAuthorStatus>()
    );

    if type_state_duration <= direct_duration * 110 / 100 && // 10%以内のオーバーヘッド
       type_state_3_duration <= direct_3_duration * 110 / 100
    {
        println!("✅ Zero-cost abstraction achieved!");
    } else {
        println!("⚠️  Performance overhead detected - optimization needed");
    }
}
