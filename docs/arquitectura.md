# Arquitectura y máquina de estados

Prism DTL separa identidad, autorización, cálculo económico y aplicación del
ledger. Una instancia de `PrismLedger` representa una red (`network_id`) y un
activo (`AssetId`). No incluye transporte, consenso ni almacenamiento
duradero: el integrador debe aportar esas capas sin convertir una respuesta
local en una confirmación global prematura.

```mermaid
flowchart TB
    subgraph Entrada
      S[Envelope firmado] --> C[crypto: identidad y firma]
      S --> E[escrow: términos y ruta]
    end
    C --> L[ledger: estado candidato]
    E --> L
    B[rebate: curva de profundidad] --> L
    A[amount: enteros acotados] --> L
    I[ids: digests por dominio] --> L
    L --> J[Diario y digest de estado]
    L --> V{Conservación}
    V -->|correcta| P[Commit atómico]
    V -->|incorrecta| R[Rechazo sin mutación]
```

## Ciclo del escrow

| Estado | Activo líquido | Valor bloqueado | Operación que avanza |
| --- | ---: | ---: | --- |
| Sin abrir | En depositante | 0 | Apertura firmada válida |
| Abierto | Debitado al depositante | Importe íntegro | Liberación firmada válida |
| Liberado | Distribuido entre destinatarios | 0 | Estado final |

`EscrowId` se deriva de depositante, beneficiario, nonce y hash de memo. El
ledger rechaza una segunda apertura del mismo identificador. El nonce de
apertura pertenece al depositante; el de liberación, al beneficiario. Un
`TxId` identifica el envelope procesado y el digest de estado permite a una
capa superior comparar réplicas.

## Bono y política de ruta

`post_operator_bond` mueve saldo líquido del operador a una reserva
inmovilizada. `set_bonded_rebate_policy` registra máximo y epoch para ese
operador una vez existe cobertura inicial. Durante una liberación compatible,
`bonded_rebate_quote` calcula el incentivo; el ledger debita el bono, acredita
al destinatario de la ruta y registra una entrada específica en el diario.
Esta transición forma parte del mismo candidato atómico que el pago del
escrow. `SupplySnapshot` distingue `bonded` del valor de escrows abiertos,
aunque ambos figuran en `locked`.

## Fronteras de atomicidad

`open_escrow` y `release_escrow` clonan el estado en un candidato, aplican la
transición y comprueban `verify_conservation` antes de reemplazar el estado
vivo. Un error de firma, balance, nonce, red, activo, ruta o conservación deja
el ledger anterior intacto. Esta propiedad se refiere al estado en memoria;
para una base de datos real hace falta transacción duradera y recuperación tras
fallo de proceso.

## Vistas de lectura

- `SettlementPath::quote(amount)` calcula la distribución bruta y neta sin
  alterar saldos. Rechaza cargos superiores al importe bruto.
- `supply_snapshot()` devuelve saldo líquido, saldo bloqueado, suministro
  declarado, reserva `bonded` y número de escrows abiertos/liberados.
- `bonded_rebate_quote(escrow_id)` devuelve la profundidad, el ratio en puntos
  básicos y el rebate previsto, o `None` si no hay política del epoch.
- `state_digest()` compromete cuentas, escrows, transacciones vistas y
  suministro, bonos y políticas. El diario no se incluye en ese digest; debe protegerse y
  archivarse por separado en la capa de persistencia.

Las vistas son herramientas de conciliación, no sustitutos de la verificación
de un envelope durante la transición definitiva.
