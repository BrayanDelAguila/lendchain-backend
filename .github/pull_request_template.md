## Descripción
<!-- Qué hace este PR y por qué -->

## Tipo de cambio
- [ ] `feature/` — nueva funcionalidad
- [ ] `fix/` — corrección de bug
- [ ] `test/` — tests nuevos o modificados
- [ ] `chore/` — configuración, deps, docs

## Checklist antes del merge
- [ ] `cargo build` pasa sin errores
- [ ] `cargo test --lib` — todos los unitarios en verde
- [ ] Sin `unwrap()` ni `expect()` fuera de tests
- [ ] Sin private keys ni secrets hardcodeados
- [ ] Variables de entorno nuevas documentadas en `.env.example`
- [ ] Migraciones SQL incluidas si hay cambios en el schema

## Tests cubiertos
<!-- Lista los tests que cubren este cambio -->

## Notas para el reviewer
<!-- Algo que el reviewer debe saber -->
