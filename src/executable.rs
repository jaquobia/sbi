/// All executables should be some variant of these:  
/// - XStarbound - will enable the removal of automatic UGC loading through the ```-noworkshop``` flag  
/// - OpenStarbound - will enable the removal of automatic UGC loading through the ```"includeUGC": false``` field in sbinit.config
/// - Vanilla - has no current method for disabling UGC content
pub enum ExecutableVariant {
    XStarbound,
    OpenStarbound,
    Vanilla
}
