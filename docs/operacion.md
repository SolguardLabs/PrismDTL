# Operación y observabilidad

Una ejecución operativa necesita un secuenciador determinista y persistencia
transaccional alrededor de `PrismLedger`. Este repositorio define la lógica de
estado, no un servicio desplegable ni una red de consenso.

## Métricas mínimas

| Métrica | Fuente | Interpretación |
| --- | --- | --- |
| `prism_supply_total` | `supply_snapshot.total` | Suministro declarado |
| `prism_supply_liquid` | `supply_snapshot.liquid` | Saldos de cuentas |
| `prism_escrow_locked` | `supply_snapshot.locked` | Obligación pendiente |
| `prism_escrow_open` | `open_escrows` | Escrows activos |
| `prism_escrow_released` | `released_escrows` | Historial liquidado |
| `prism_transition_rejected_total` | Errores tipados | Rechazos por causa |

Tras cada commit, comparar `liquid + locked` con `total`. Alertar ante
cualquier divergencia, cambio inesperado de suministro o secuencia de nonces
que no corresponda al diario. El crecimiento de rechazos de firma o de
transacción duplicada puede reflejar integración defectuosa o abuso de la API;
no prueba por sí solo un incidente.

## Recuperación

1. Detener la admisión de nuevos envelopes y conservar el estado actual.
2. Guardar snapshot, diario, digest, versión del binario y orden de entrada.
3. Verificar la última transacción confirmada y reproducir el diario en una
   instancia aislada.
4. Comparar `state_digest` y `supply_snapshot` entre original y reproducción.
5. Reabrir admisión solo cuando la causa y el punto de recuperación sean
   inequívocos.

La copia del estado en memoria es una frontera de atomicidad del crate. El
operador debe aportar un mecanismo equivalente para escrituras duraderas; de
otro modo, un reinicio entre confirmación y persistencia puede causar estados
divergentes.

## Publicación

El pipeline ejecuta `cargo fmt`, compilación, tests y Clippy, además de
formato, sintaxis y tests Bun. Un release debe referenciar un commit único en
`main`, `production` y el tag anotado. El workflow de integridad verifica
esta igualdad y la existencia del banner y la documentación versionados.
