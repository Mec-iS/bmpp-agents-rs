use anyhow::Result;
use bmpp_agents::transpiler::{parser::parse_source, codegen::BmppCodeGenerator};

#[test]
fn test_protocol_composition_basic() -> Result<()> {
    let bmpp_source = r#"
        Pack <Protocol>("Pack protocol for packaging items") {
            roles
                W <Agent>("Warehouse"),
                P <Agent>("Packer"),
                S <Agent>("Scanner")
            parameters
                ID <String>("unique identifier"),
                order <String>("order details"),
                tag <String>("package tag"),
                package <String>("packaged item")
            
            W -> P: Pack <Action>("pack the order")[out ID, out order]
            P -> W: Tag <Action>("tag the package")[in ID, in order, out tag]
            P -> S: WriteTag <Action>("write tag data")[in ID, in order, in tag]
            S -> P: TagWritten <Action>("confirm tag written")[in ID, in tag, out package]
        }
        
        Logistics <Protocol>("Logistics protocol using Pack") {
            roles
                M <Agent>("Manager"),
                W <Agent>("Warehouse"),
                P <Agent>("Packer"),
                L <Agent>("Loader"),
                S <Agent>("Scanner"),
                C <Agent>("Courier")
            parameters
                ID <String>("unique identifier"),
                order <String>("order details"),
                delivery <String>("delivery confirmation")
            
            M -> W: NotifyOrder <Action>("notify of new order")[out ID, out order]
            Pack(W, P, S, in ID, in order, out tag, out package)
            W -> M: Deliver <Action>("confirm delivery")[in ID, in package, out delivery]
        }
    "#;
    
    let ast = parse_source(bmpp_source)?;
    let code_generator = BmppCodeGenerator::new();
    let generated_code = code_generator.generate(&ast)?;
    
    // Verify that the generated code contains both protocols
    assert!(generated_code.contains("pub struct PackProtocol"));
    assert!(generated_code.contains("pub struct LogisticsProtocol"));
    assert!(generated_code.contains("pub fn pack"));
    
    Ok(())
}


#[test]
fn test_nested_protocol_composition() -> Result<()> {
    let bmpp_source = r#"
        Load <Protocol>("Load protocol") {
            roles
                W <Agent>("Warehouse"),
                L <Agent>("Loader"),
                S <Agent>("Scanner"),
                C <Agent>("Courier")
            parameters
                ID <String>("identifier"),
                order <String>("order"),
                tag <String>("tag"),
                route <String>("route")
            
            W -> L: Load <Action>("load item")[in ID, in order, in tag]
            L -> C: GetVehicle <Action>("get vehicle")[in ID, in order, out vehicle]
            C -> W: Enroute <Action>("confirm en route")[in ID, in vehicle, out route]
        }
        
        Pack <Protocol>("Pack protocol") {
            roles
                W <Agent>("Warehouse"),
                P <Agent>("Packer"),
                S <Agent>("Scanner")
            parameters
                ID <String>("identifier"),
                order <String>("order"),
                tag <String>("tag"),
                package <String>("package")
            
            W -> P: Pack <Action>("pack item")[in ID, in order]
            P -> W: Packed <Action>("confirm packed")[in ID, out tag, out package]
        }
        
        LogisticsWithNestedComposition <Protocol>("Logistics with nested composition") {
            roles
                M <Agent>("Manager"),
                W <Agent>("Warehouse"),
                P <Agent>("Packer"),
                L <Agent>("Loader"),
                S <Agent>("Scanner"),
                C <Agent>("Courier")
            parameters
                ID <String>("identifier"),
                order <String>("order"),
                delivery <String>("delivery")
            
            M -> W: NotifyOrder <Action>("notify order")[out ID, out order]
            Pack[W, P, S, in ID, in order, out tag, out package]
            Load[W, L, S, C, in ID, in order, in tag, out route]
            W -> M: Deliver <Action>("deliver")[in ID, in route, out delivery]
        }
    "#;
    
    let ast = parse_source(bmpp_source)?;
    let code_generator = BmppCodeGenerator::new();
    let generated_code = code_generator.generate(&ast)?;
    
    // Verify all protocols are generated with proper composition
    assert!(generated_code.contains("pub struct LoadProtocol"));
    assert!(generated_code.contains("pub struct PackProtocol"));  
    assert!(generated_code.contains("pub struct LogisticsWithNestedCompositionProtocol"));
    
    Ok(())
}
