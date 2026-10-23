# Modelo económico y conciliación

La unidad base es un entero `u128` acotado por `MAX_PRISM_SUPPLY` (10¹³
unidades). Prism no interpreta decimales ni precios de mercado: un adaptador
puede presentar seis o dieciocho decimales, pero debe hacerlo de forma
uniforme en depósitos, rutas, comisiones y reportes.

## Distribución de una liberación

Para importe bruto `G`, comisión de operador `O` y rebate `R`:

```text
C = O + R                       (suma comprobada)
B = G - C                       (resta comprobada, requiere C <= G)
B + O + R = G                   (conservación por liquidación)
```

`SettlementQuote` devuelve los cinco valores `gross_amount`,
`beneficiary_amount`, `operator_fee`, `rebate_amount` y `total_charges`.
`rebate_recipient` ausente envía el importe de rebate al beneficiario; por
tanto, la recepción efectiva del beneficiario sería `B + R` en ese caso.
La cuota se calcula en apertura y se recalcula en liberación.

## Incentivo respaldado por bono

Una política de operador puede añadir un rebate `I` financiado por un bono
inmovilizado. `I` no procede de `G` y no se mezcla con `R`. La política está
vinculada al `routing_epoch` y define un máximo `M`. Para profundidad elegible
`D` e importe bruto `G`, la curva usa aritmética entera:

```text
r = floor(10.000 × D / (D + G))           puntos básicos
I = floor(M × r² / 10.000²)
I <= M
```

Por ejemplo, una profundidad observada de 1.500, un escrow de 1.000 y un
máximo de 400 producen `r = 6.000` y `I = 144`. La operación confirma el abono
solo si el bono puede soportarlo; los rechazos conservan el estado anterior.

```mermaid
flowchart LR
    P[Política por operador y epoch] --> Q[Cotización entera]
    D[Profundidad de ruta] --> Q
    G[Bruto del escrow] --> Q
    Q -->|I| R[Destinatario]
    C[Bono operador] -->|débito I| R
    G -->|B + O + R fijo| S[Liquidación escrow]
```

| Concepto | Ruta directa | Ruta con operador e integrador |
| --- | ---: | ---: |
| Escrow bloqueado | 600 | 600 |
| Comisión operador | 0 | 10 |
| Rebate | 0 | 40 |
| Beneficiario neto | 600 | 550 |
| Total acreditado | 600 | 600 |

## Balance global

```mermaid
flowchart LR
    T[Suministro total] --> A[Saldos líquidos]
    T --> E[Escrows abiertos]
    T --> C[Bonos de operador]
    E -->|liberación| A
    C -->|rebate respaldado| A
    X[Escrows liberados] -.->|no forman parte del bloqueado| E
```

La invariante global es `total_supply = Σ account.balance + Σ
escrow.amount + Σ operator.bond` para escrows no liberados. Un escrow liberado permanece como
registro histórico, pero su importe ya está en los saldos de destinatarios.
`verify_conservation` detecta inflación, pérdida contable o doble cómputo; no
determina si una ruta satisface acuerdos comerciales fuera del ledger.

## Cotas y rechazo

- `Amount::new` rechaza valores superiores a 10¹³.
- `checked_add` y `checked_sub` rechazan overflow y underflow.
- Una ruta con `O + R > G` se rechaza antes de debitar al depositante.
- La financiación génesis aumenta saldo y suministro conjuntamente. No es una
  API de emisión pública: el servicio que la use debe limitarla a su fase de
  inicialización y registrar su procedencia.

Para métricas financieras externas, registrar por transacción el importe
bruto, los tres abonos, el valor bloqueado antes/después y el digest de estado.
No calcular PnL a partir de una vista parcial de cuentas.
