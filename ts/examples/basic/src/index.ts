import { f32 } from 'flecs/emscripten'
import { Type, Component, Entity, World } from 'flecs/ecs'

// Define component
class Position extends Component {
    public x: f32 = 0
    public y: f32 = 0
}

class Velocity extends Component {
    public x: f32 = 0
    public y: f32 = 0
}

// Register component
World.registerComponent(new Position({ x: Type.F32, y: Type.F32 }))
World.registerComponent(new Velocity({ x: Type.F32, y: Type.F32 }))

// Create entities
const entities = 5
for (let i = 0; i < entities; i++) {
    const entity = new Entity()
    entity.add(new Position())
}

// Query
const query = World.query(Position)
// Start next cycle of iteration
query.iter()
// Iterate
while (query.next()) {
    const positions = query.field(Position)
    // Iterate through postions
    positions.forEach((position: Position) => {
        // Modify position
        position.x = 10
        position.y = 10
    })
}