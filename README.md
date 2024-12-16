# dfe
DFE - Documentos Fiscais Eletrônicos Brasileiros.

Se caiu de paraquedas aqui, sim, o código está um C#! Tenho que refatorar muito ainda. 

Estou escrevendo este projeto por necessidade própria e espero terminar todos os recursos necessários, a intenção é escrever uma API REST que consuma essa crate.

Porque RUST?
Porque estou aprendendo a linguagem KKK.

## Instalação:
-> É necessário instalar o OPENSSL no seu ambiente de desenvolvimento. No ambiente Windows (como o meu), instale também o Pear.   
-> Copie a pasta dfe que contém os arquivos de SCHEMA para o diretório do seu executável.   
-> cargo add dfe  

## Exemplo de Uso: Emitir NFe

```rust
use dfe::nfe::autorizacao::emit;
use dfe::nfe::types::autorizacao4::*;

let teste = emit(NFe {
    cert_path: "D:/Projetos/cert.pfx".to_string(),
    cert_pass: "1234".to_string(),
    ide: Ide {
        c_uf: 35,
        serie: 1,
        n_nf: 35,
        c_mun_fg: 3550308,
        tp_emis: 1,
        tp_amb: 2,
        ind_final: 1,
        ind_pres: 1,
        mod_: 55,
        tp_imp: 1,
        ..Default::default()
    },
    emit: Emit {
        cnpj: Some("11111111111111".to_string()),
        ie: Some(111111111111),
        crt: 3,
        x_nome: "EMPRESA DE TESTE".to_string(),
        x_fant: Some("TESTANDO EMPREENDIMENTOS".to_string()),
        x_lgr: "RUA TESTE".to_string(),
        nro: "123".to_string(),
        x_bairro: "CENTRO".to_string(),
        c_mun: 3529906,
        x_mun: "SÃO PAULO".to_string(),
        uf: "SP".to_string(),
        cep: 10000000,
        ..Default::default()
    },
    dest: Dest {
        cpf: Some("07068093868".to_string()),
        //cnpj: Some("56196407000190".to_string()), // com ie
        //cnpj: Some("46395000000139".to_string()), // sem ie
        x_nome: Some("NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string()),
        x_lgr: Some("RUA TESTE".to_string()),
        nro: Some("123".to_string()),
        x_bairro: Some("CENTRO".to_string()),
        c_mun: Some(3550308),
        x_mun: Some("SÃO PAULO".to_string()),
        uf: Some("SP".to_string()),
        cep: Some(11850000),
        //c_pais: Some("1058".to_string()),
        //x_pais: Some("BRASIL".to_string()),
        //fone: Some("11999999999".to_string()),
        ind_ie_dest: Some(9),
        //ie: Some("150344006118".to_string()),
        ..Default::default()
    },
    det: vec![
        Det {
            c_prod: "123456".to_string(),
            x_prod: "NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 1.0,
            v_un_com: 10.0,
            v_prod: 10.0,
            u_trib: "CX".to_string(),
            q_trib: 1.0,
            v_un_trib: "10.00".to_string(),
            ind_tot: 1,
            // TODO: Dispobilizar todos os tipos de ICMS
            // Disponivel: -> ICMS40 ou ICMSSN102
            // orig -> 0
            // CST -> 41
            // csosn -> 102
            icms: "ICMS40".to_string(),
            // TODO: Dispobilizar
            pis: "PISNT".to_string(),
            // TODO: Dispobilizar
            cofins: "COFINSNT".to_string(),
            ..Default::default()
        },
        Det {
            c_prod: "123456".to_string(),
            x_prod: "PRODUTO TESTE 2".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 2.0,
            v_un_com: 10.0,
            v_prod: 20.0,
            u_trib: "CX".to_string(),
            q_trib: 2.0,
            v_un_trib: "10.00".to_string(),
            ind_tot: 1,
            icms: "ICMS40".to_string(),
            pis: "PISNT".to_string(),
            cofins: "COFINSNT".to_string(),
            ..Default::default()
        },
    ],
    total: Total {
        v_bc: 0.0,
        v_icms: 0.0,
        v_icms_deson: 0.0,
        v_fcpuf_dest: 0.0,
        v_icms_uf_dest: 0.0,
        v_icms_uf_remet: 0.0,
        v_fcp: 0.0,
        v_bc_st: 0.0,
        v_st: 0.0,
        v_fcpst: 0.0,
        v_fcpst_ret: 0.0,
        v_prod: 30.0,
        v_frete: 0.0,
        v_seg: 0.0,
        v_desc: 0.0,
        v_ii: 0.0,
        v_ipi: 0.0,
        v_ipi_devol: 0.0,
        v_pis: 0.0,
        v_cofins: 0.0,
        v_outro: 0.0,
        v_nf: 30.0,
        v_tot_trib: 0.0,
    },
    transp: Transp {
        mod_frete: 0,
        ..Default::default()
    },
    pag: Pag {
        t_pag: "01".to_string(),
        v_pag: 30.0,
    },
    inf_adic: None,
})
.await;

if let Err(e) = teste {
    println!("Erro: {:?}", e);
} else {
    if let Ok(response) = teste {
        println!("Resposta: Ok(Response {{");
        println!("    inf_prot: InfProt {{");
        println!("        tp_amb: {},", response.protocolo.inf_prot.tp_amb);
        println!("        ver_aplic: \"{}\",", response.protocolo.inf_prot.ver_aplic);
        println!("        ch_nfe: \"{}\",", response.protocolo.inf_prot.ch_nfe);
        println!("        dh_recbto: \"{}\",", response.protocolo.inf_prot.dh_recbto);
        println!("        n_prot: {:?},", response.protocolo.inf_prot.n_prot);
        println!("        dig_val: {:?},", response.protocolo.inf_prot.dig_val);
        println!("        c_stat: {},", response.protocolo.inf_prot.c_stat);
        println!("        x_motivo: \"{}\"", response.protocolo.inf_prot.x_motivo);
        println!("    }}");
        println!("}}");

        // print xml
        println!("XML: {:?}", response.xml);
    }
}
```
# Exemplo de uso: Status do Serviço 
Webservice SP Produção
```rust
use dfe::nfe::service_status;
use dfe::nfe::types::config::*;
// TODO mudar o tipo para receber Estado, Ambiente, NFe ou NFCe Homologação ou Produção
let teste = service_status(Use::ManualConfig(Fields {
    cert_path: "D:/Projetos/cert.pfx".to_string(),
    cert_pass: Password::Phrase("4101".to_string()),
    federative_unit: "SP".to_string(),
    environment: Environment::Homologation,
}))
.await;
if teste.is_err() {
    println!("Error test_service_status_custom_pass:{:?}", teste.err());
    assert!(false);
    return;
}
let teste = teste.unwrap();

println!("tp_amb: {}", teste.tp_amb);
println!("ver_aplic: {}", teste.ver_aplic);
println!("c_stat: {}", teste.c_stat);
println!("x_motivo: {}", teste.x_motivo);
println!("c_uf: {}", teste.c_uf);
println!("dh_recbto: {}", teste.dh_recbto);
println!("t_med: {}", teste.t_med);
```

## Notas importantes:

Este software está em desenvolvimento e não deve ser usado em produção a não ser que você saiba o que está fazendo.

Este software é distribuído sem garantia e sem nenhuma responsabilidade do autor.
Autor: Gustavo Ota - Gravis Tec
Para doações entre em contato WhatsApp: +55 13 99782 1459

