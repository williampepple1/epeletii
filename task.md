# Tasks - Epeletii Improvements

## Backend
- [x] Implement tone-stripping normalization in `backend/src/dictionary.rs`
- [x] Refactor `Dictionary` struct and SQLite loading to store `WordDetail` lists
- [x] Implement `lookup` method in `Dictionary`
- [x] Add `LookupWord` and `WordDefinition` to `backend/src/protocol.rs`
- [x] Handle `LookupWord` in `backend/src/main.rs`
- [x] Run backend tests to verify
- [x] Merge underdots into standard tiles in `tiles.rs` and update validation matching to match regardless of underdots/tone marks.

## Frontend
- [x] Update types in `frontend/src/types/game.ts`
- [x] Add `lookupWord` and `shuffleRack` actions/handlers in `frontend/src/store/gameStore.ts`
- [x] Create `DictionarySidebar` component for word meanings & lookup
- [x] Integrate sidebar in `frontend/src/app/page.tsx`
- [x] Add Shuffle button and styling to `frontend/src/components/TileRack.tsx`
- [x] Refactor `frontend/src/components/GameBoard.tsx` to support premium design and keyboard controls
- [x] Initialize theme on client mount in `frontend/src/app/page.tsx` and clean up `frontend/src/app/layout.tsx`
- [x] Fix drag-and-drop bug (replace button with div in TileRack, decouple selection from placement in store)

## Verification
- [x] Verify build and tests pass (npx tsc, npm run build, cargo test)
- [x] Walkthrough documentation
