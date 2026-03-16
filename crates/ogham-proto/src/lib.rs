// @generated
pub mod ogham {
    #[cfg(feature = "ogham-common")]
    // @@protoc_insertion_point(attribute:ogham.common)
    pub mod common {
        include!("ogham/common/ogham.common.rs");
        // @@protoc_insertion_point(ogham.common)
    }
    #[cfg(feature = "ogham-compiler")]
    // @@protoc_insertion_point(attribute:ogham.compiler)
    pub mod compiler {
        include!("ogham/compiler/ogham.compiler.rs");
        // @@protoc_insertion_point(ogham.compiler)
    }
    #[cfg(feature = "ogham-ir")]
    // @@protoc_insertion_point(attribute:ogham.ir)
    pub mod ir {
        include!("ogham/ir/ogham.ir.rs");
        // @@protoc_insertion_point(ogham.ir)
    }
}