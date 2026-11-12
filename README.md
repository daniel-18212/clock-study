# Clock Study

Aplicativo desktop de foco para Windows 11. Pomodoro, cronômetro, temporizador e janela flutuante sempre acima das outras janelas.

## Compilar no Windows

Instale o [Rust com rustup](https://rustup.rs/) e o [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). No instalador do Visual Studio, selecione **Desenvolvimento para desktop com C++**, com MSVC e SDK do Windows. Use Node.js 22 LTS e o WebView2 do Windows. Veja os [pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/).

```powershell
npm install
npm run tauri dev
npm run tauri build -- --no-bundle
```

O executável de compilação fica em `src-tauri/target/release/clock-study.exe`. O Windows pode bloquear a execução desse binário sem assinatura quando o Controle Inteligente de Aplicativos estiver ativo.

## Gerar e publicar o MSIX

1. Crie o produto **MSIX/PWA** no [Partner Center](https://partner.microsoft.com/dashboard). Abra a seção **Identidade do produto** e copie o nome da identidade, o publicador e o nome de exibição do publicador.
2. No GitHub, em **Settings → Secrets and variables → Actions → Variables**, adicione `STORE_IDENTITY_NAME`, `STORE_PUBLISHER` e `STORE_PUBLISHER_DISPLAY_NAME`.
3. Rode o workflow **Clock Study Windows** com **Preparar MSIX para submissão à Microsoft Store** marcado. Também é possível empacotar localmente com o script `packaging/windows/pack-msix.ps1`, passando esses três campos pelos parâmetros.
4. Envie o MSIX para certificação no Partner Center. A Store assina o pacote durante a publicação. Valide a instalação da versão publicada com o Controle Inteligente de Aplicativos ativo.

[Documentação da Microsoft sobre assinatura pela Store](https://learn.microsoft.com/windows/apps/publish/faq/get-started-with-the-microsoft-store)

## Verificar no Windows 11

- Inicie um Pomodoro, pause, retome e reinicie. Edite as três durações e o número de blocos.
- Defina um temporizador de 10 segundos, minimize o app e confira a notificação ao terminar.
- Registre voltas no cronômetro e abra a lista. Reinicie e confira que a lista foi limpa.
- Acione **Flutuar**, abra outro aplicativo e confira se o relógio permanece acima. Arraste a janela e redimensione pelos cantos. Volte à janela principal com a contagem ativa.
- Altere os tamanhos e os tempos, feche e reabra. Verifique se as preferências foram recuperadas.
