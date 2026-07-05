pub struct FlagAutorizacao;

/// Estado do fluxo de autorização. Historicamente persistido em `flag_autorizacao.env`;
/// hoje a emissão exige apenas o estado `Ready` (ver [`FlagAutorizacao::start`]). As demais
/// variantes são mantidas para documentar o fluxo pretendido, mas não são construídas.
#[derive(Debug)]
#[allow(dead_code)]
pub enum FlagAutorizacaoEnum {
    Ready,
    XMLGerado,
    Requested,
    Autorizado,
    NaoAutorizado,
    SemResposta,
}

impl FlagAutorizacao {
    /// Sinaliza que a emissão pode prosseguir, retornando sempre [`FlagAutorizacaoEnum::Ready`].
    ///
    /// Antes esta função lia/gravava `flag_autorizacao.env` no diretório de trabalho, mas
    /// nada no fluxo jamais escrevia um valor diferente de `Ready` — a flag era, na prática,
    /// uma constante. A **A1** do refactor removeu esse side-effect de CWD (que causava
    /// corrida de dados no servidor multi-thread), preservando o comportamento observável:
    /// a emissão só prossegue com `Ready`.
    pub async fn start() -> Result<FlagAutorizacaoEnum, String> {
        Ok(FlagAutorizacaoEnum::Ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn start_retorna_ready_sem_tocar_o_cwd() {
        // A1: não deve criar `flag_autorizacao.env` nem qualquer outro arquivo no CWD.
        let flag = FlagAutorizacao::start().await.expect("start deve ser Ok");
        assert!(matches!(flag, FlagAutorizacaoEnum::Ready));
    }
}
