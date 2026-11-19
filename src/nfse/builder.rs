use crate::error::{DfeError, Result};
use crate::{validate_cnpj, validate_cpf};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate};
use quick_xml::events::{BytesDecl, BytesText, Event};
use quick_xml::Writer;
use rust_decimal::Decimal;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ambiente {
    Producao,
    ProducaoRestrita,
}

#[derive(Clone, Debug)]
pub enum Documento {
    /// Onze dígitos, sem máscara.
    Cpf(String),
    /// Quatorze caracteres, sem máscara. Letras são aceitas em produção restrita.
    Cnpj(String),
}

#[derive(Clone, Debug)]
pub struct IdentificacaoDps {
    pub municipio_emissao: String,
    /// Série para aplicativo próprio ou API, de 1 a 49.999.
    pub serie: u32,
    pub numero: u64,
    pub emitida_em: DateTime<FixedOffset>,
    pub competencia: NaiveDate,
    pub versao_aplicativo: String,
}

#[derive(Clone, Copy, Debug)]
pub enum RegimeTributario {
    Mei,
    /// Tributos federais e ISSQN apurados no Simples Nacional.
    SimplesNacional,
}

#[derive(Clone, Debug)]
pub struct Prestador {
    pub documento: Documento,
    pub inscricao_municipal: Option<String>,
    pub regime: RegimeTributario,
}

#[derive(Clone, Debug)]
pub struct Tomador {
    pub documento: Documento,
    pub nome: String,
}

#[derive(Clone, Debug)]
pub struct Servico {
    pub municipio_prestacao: String,
    pub codigo_tributacao_nacional: String,
    pub codigo_tributacao_municipal: Option<String>,
    pub codigo_nbs: Option<String>,
    pub descricao: String,
    pub valor: Decimal,
}

#[derive(Clone, Copy, Debug)]
pub enum TotalTributos {
    NaoInformado,
    PercentualSimplesNacional(Decimal),
}

#[derive(Clone, Copy, Debug)]
pub struct Tributacao {
    pub total: TotalTributos,
}

#[derive(Clone, Debug)]
pub struct DpsGerada {
    ambiente: Ambiente,
    id: String,
    xml: String,
}

impl DpsGerada {
    pub fn ambiente(&self) -> Ambiente {
        self.ambiente
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn xml(&self) -> &str {
        &self.xml
    }
}

/// Builder para uma DPS regular, de serviço nacional tributável sem retenção
/// de ISSQN, emitida pelo prestador MEI ou optante do Simples Nacional.
/// A classificação do serviço e o enquadramento fiscal vêm do chamador.
#[derive(Default)]
pub struct NfseBuilder {
    ambiente: Option<Ambiente>,
    identificacao: Option<IdentificacaoDps>,
    prestador: Option<Prestador>,
    tomador: Option<Tomador>,
    servico: Option<Servico>,
    tributacao: Option<Tributacao>,
    desconto_incondicionado: Decimal,
}

impl NfseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ambiente(mut self, ambiente: Ambiente) -> Self {
        self.ambiente = Some(ambiente);
        self
    }

    pub fn identificacao(mut self, identificacao: IdentificacaoDps) -> Self {
        self.identificacao = Some(identificacao);
        self
    }

    pub fn prestador(mut self, prestador: Prestador) -> Self {
        self.prestador = Some(prestador);
        self
    }

    pub fn tomador(mut self, tomador: Tomador) -> Self {
        self.tomador = Some(tomador);
        self
    }

    pub fn servico(mut self, servico: Servico) -> Self {
        self.servico = Some(servico);
        self
    }

    pub fn tributacao(mut self, tributacao: Tributacao) -> Self {
        self.tributacao = Some(tributacao);
        self
    }

    /// Desconto em reais. Zero omite o grupo de descontos da DPS.
    pub fn desconto_incondicionado(mut self, valor: Decimal) -> Self {
        self.desconto_incondicionado = valor;
        self
    }

    pub fn gerar_dps(self) -> Result<DpsGerada> {
        let ambiente = self.ambiente.ok_or_else(|| invalido("ambiente"))?;
        let ident = self
            .identificacao
            .ok_or_else(|| invalido("identificação"))?;
        let prestador = self.prestador.ok_or_else(|| invalido("prestador"))?;
        let servico = self.servico.ok_or_else(|| invalido("serviço"))?;
        let tributacao = self.tributacao.ok_or_else(|| invalido("tributação"))?;

        digitos("cLocEmi", &ident.municipio_emissao, 7)?;
        if !(1..=49_999).contains(&ident.serie) {
            return Err(DfeError::Validacao(
                "DPS: a série para emissão via API deve estar entre 1 e 49999 (E0010)".into(),
            ));
        }
        if !(1..=999_999_999_999_999).contains(&ident.numero) {
            return Err(invalido("nDPS"));
        }
        let offset = ident.emitida_em.offset().local_minus_utc();
        if !(2000..=2099).contains(&ident.emitida_em.year())
            || !(-11 * 3600..=12 * 3600).contains(&offset)
            || offset % 3600 != 0
            || ident.emitida_em.timestamp_subsec_nanos() != 0
        {
            return Err(invalido("dhEmi"));
        }
        if !(2000..=2099).contains(&ident.competencia.year()) {
            return Err(invalido("dCompet"));
        }
        if ident.competencia > ident.emitida_em.date_naive() {
            return Err(DfeError::Validacao(
                "DPS: a competência (dCompet) não pode ser posterior à data de emissão (E0015)"
                    .into(),
            ));
        }
        texto("verAplic", &ident.versao_aplicativo, 20)?;
        prestador.documento.validar(ambiente)?;
        if let Some(im) = &prestador.inscricao_municipal {
            texto("IM", im, 15)?;
        }
        if let Some(tomador) = &self.tomador {
            tomador.documento.validar(ambiente)?;
            texto("xNome", &tomador.nome, 300)?;
        }
        digitos("cLocPrestacao", &servico.municipio_prestacao, 7)?;
        digitos("cTribNac", &servico.codigo_tributacao_nacional, 6)?;
        if let Some(codigo) = &servico.codigo_tributacao_municipal {
            digitos("cTribMun", codigo, 3)?;
        }
        if let Some(codigo) = &servico.codigo_nbs {
            digitos("cNBS", codigo, 9)?;
        }
        texto("xDescServ", &servico.descricao, 2000)?;
        decimal("vServ", servico.valor, 1_000_000_000_000_000)?;
        if servico.valor.is_zero() {
            return Err(DfeError::Validacao(
                "DPS: o valor do serviço (vServ) deve ser maior que zero".into(),
            ));
        }
        let desconto = self.desconto_incondicionado;
        decimal("vDescIncond", desconto, 1_000_000_000_000_000)?;
        if desconto >= servico.valor {
            return Err(DfeError::Validacao(
                "DPS: o desconto incondicionado (vDescIncond) deve ser menor que o valor do serviço"
                    .into(),
            ));
        }
        if let TotalTributos::PercentualSimplesNacional(percentual) = tributacao.total {
            if !matches!(prestador.regime, RegimeTributario::SimplesNacional) {
                return Err(invalido("pTotTribSN requer apuração pelo Simples Nacional"));
            }
            decimal("pTotTribSN", percentual, 100)?;
        }

        let (tipo, inscricao) = match &prestador.documento {
            Documento::Cpf(cpf) => (1, format!("000{cpf}")),
            Documento::Cnpj(cnpj) => (2, cnpj.clone()),
        };
        let id = format!(
            "DPS{}{tipo}{inscricao}{:05}{:015}",
            ident.municipio_emissao, ident.serie, ident.numero
        );
        let mut xml = Writer::new(Vec::new());
        xml.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
        xml.create_element("DPS")
            .with_attribute(("xmlns", "http://www.sped.fazenda.gov.br/nfse"))
            .with_attribute(("versao", "1.01"))
            .write_inner_content(|xml| {
                xml.create_element("infDPS")
                    .with_attribute(("Id", id.as_str()))
                    .write_inner_content(|xml| {
                        tag(
                            xml,
                            "tpAmb",
                            match ambiente {
                                Ambiente::Producao => "1",
                                Ambiente::ProducaoRestrita => "2",
                            },
                        )?;
                        tag(
                            xml,
                            "dhEmi",
                            &ident.emitida_em.format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
                        )?;
                        tag(xml, "verAplic", &ident.versao_aplicativo)?;
                        tag(xml, "serie", &ident.serie.to_string())?;
                        tag(xml, "nDPS", &ident.numero.to_string())?;
                        tag(xml, "dCompet", &ident.competencia.to_string())?;
                        tag(xml, "tpEmit", "1")?;
                        if ambiente == Ambiente::ProducaoRestrita {
                            tag(xml, "finNFSe", "0")?;
                        }
                        tag(xml, "cLocEmi", &ident.municipio_emissao)?;
                        escrever_prestador(xml, &prestador)?;
                        if let Some(tomador) = &self.tomador {
                            xml.create_element("toma").write_inner_content(|xml| {
                                tomador.documento.escrever(xml)?;
                                tag(xml, "xNome", &tomador.nome)
                            })?;
                        }
                        escrever_servico(xml, ambiente, &servico)?;
                        escrever_valores(xml, servico.valor, desconto, tributacao.total)
                    })?;
                Ok(())
            })?;
        let xml = String::from_utf8(xml.into_inner()).map_err(|e| DfeError::Xml(e.to_string()))?;
        Ok(DpsGerada { ambiente, id, xml })
    }
}

impl Documento {
    fn validar(&self, ambiente: Ambiente) -> Result<()> {
        match self {
            Self::Cpf(cpf)
                if cpf.len() == 11
                    && cpf.bytes().all(|c| c.is_ascii_digit())
                    && validate_cpf(cpf) =>
            {
                Ok(())
            }
            Self::Cnpj(cnpj)
                if cnpj.len() == 14
                    && cnpj.bytes().all(|c| {
                        c.is_ascii_digit()
                            || (ambiente == Ambiente::ProducaoRestrita && c.is_ascii_uppercase())
                    })
                    && validate_cnpj(cnpj) =>
            {
                Ok(())
            }
            Self::Cpf(_) => Err(invalido("CPF")),
            Self::Cnpj(_) => Err(invalido("CNPJ para o ambiente selecionado")),
        }
    }

    fn escrever(&self, xml: &mut Writer<Vec<u8>>) -> io::Result<()> {
        match self {
            Self::Cpf(cpf) => tag(xml, "CPF", cpf),
            Self::Cnpj(cnpj) => tag(xml, "CNPJ", cnpj),
        }
    }
}

fn escrever_prestador(xml: &mut Writer<Vec<u8>>, prestador: &Prestador) -> io::Result<()> {
    xml.create_element("prest").write_inner_content(|xml| {
        prestador.documento.escrever(xml)?;
        if let Some(im) = &prestador.inscricao_municipal {
            tag(xml, "IM", im)?;
        }
        xml.create_element("regTrib").write_inner_content(|xml| {
            match prestador.regime {
                RegimeTributario::Mei => tag(xml, "opSimpNac", "2")?,
                RegimeTributario::SimplesNacional => {
                    tag(xml, "opSimpNac", "3")?;
                    tag(xml, "regApTribSN", "1")?;
                }
            }
            tag(xml, "regEspTrib", "0")
        })?;
        Ok(())
    })?;
    Ok(())
}

fn escrever_servico(
    xml: &mut Writer<Vec<u8>>,
    ambiente: Ambiente,
    servico: &Servico,
) -> io::Result<()> {
    xml.create_element("serv").write_inner_content(|xml| {
        xml.create_element("locPrest")
            .write_inner_content(|xml| tag(xml, "cLocPrestacao", &servico.municipio_prestacao))?;
        xml.create_element("cServ").write_inner_content(|xml| {
            tag(xml, "cTribNac", &servico.codigo_tributacao_nacional)?;
            if let Some(codigo) = &servico.codigo_tributacao_municipal {
                tag(xml, "cTribMun", codigo)?;
            }
            if ambiente == Ambiente::Producao {
                tag(xml, "xDescServ", &servico.descricao)?;
            }
            if let Some(codigo) = &servico.codigo_nbs {
                tag(xml, "cNBS", codigo)?;
            }
            if ambiente == Ambiente::ProducaoRestrita {
                tag(xml, "xDescServ", &servico.descricao)?;
            }
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(())
}

fn escrever_valores(
    xml: &mut Writer<Vec<u8>>,
    valor: Decimal,
    desconto: Decimal,
    total: TotalTributos,
) -> io::Result<()> {
    xml.create_element("valores").write_inner_content(|xml| {
        xml.create_element("vServPrest")
            .write_inner_content(|xml| tag(xml, "vServ", &format!("{valor:.2}")))?;
        if !desconto.is_zero() {
            xml.create_element("vDescCondIncond")
                .write_inner_content(|xml| tag(xml, "vDescIncond", &format!("{desconto:.2}")))?;
        }
        xml.create_element("trib").write_inner_content(|xml| {
            xml.create_element("tribMun").write_inner_content(|xml| {
                tag(xml, "tribISSQN", "1")?;
                tag(xml, "tpRetISSQN", "1")
            })?;
            xml.create_element("totTrib")
                .write_inner_content(|xml| match total {
                    TotalTributos::NaoInformado => tag(xml, "indTotTrib", "0"),
                    TotalTributos::PercentualSimplesNacional(p) => {
                        tag(xml, "pTotTribSN", &format!("{p:.2}"))
                    }
                })?;
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(())
}

fn tag(xml: &mut Writer<Vec<u8>>, nome: &str, valor: &str) -> io::Result<()> {
    xml.create_element(nome)
        .write_text_content(BytesText::new(valor))?;
    Ok(())
}

fn invalido(campo: &str) -> DfeError {
    DfeError::Validacao(format!("DPS: {campo} ausente ou inválido"))
}

fn digitos(campo: &str, valor: &str, tamanho: usize) -> Result<()> {
    if valor.len() != tamanho || !valor.bytes().all(|c| c.is_ascii_digit()) {
        return Err(invalido(campo));
    }
    Ok(())
}

fn texto(campo: &str, valor: &str, max: usize) -> Result<()> {
    if valor.trim().is_empty() || valor.chars().count() > max
        || !valor.chars().all(|c| matches!(c,
            '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'
        ))
    {
        return Err(invalido(campo));
    }
    Ok(())
}

fn decimal(campo: &str, valor: Decimal, limite: u64) -> Result<()> {
    if valor < Decimal::ZERO || valor >= Decimal::from(limite) || valor.normalize().scale() > 2 {
        return Err(invalido(campo));
    }
    Ok(())
}
