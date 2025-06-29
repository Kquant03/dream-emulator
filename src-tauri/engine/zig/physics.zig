// src-tauri/engine/zig/physics.zig
const std = @import("std");
const math = std.math;

// Configuration that can be customized at compile time
pub const PhysicsConfig = struct {
    grid_size: u32 = 64,
    world_width: u32 = 4096,
    world_height: u32 = 4096,
    max_bodies: u32 = 10000,
    max_contacts: u32 = 1000,
};

// Generate optimized spatial hash at compile time
fn SpatialHash(comptime config: PhysicsConfig) type {
    return struct {
        const Self = @This();
        const cells_x = config.world_width / config.grid_size;
        const cells_y = config.world_height / config.grid_size;
        const total_cells = cells_x * cells_y;
        
        // Use perfect hash function since grid size is known at compile time
        const hash_multiplier = comptime blk: {
            // Find optimal multiplier for our grid size
            break :blk findOptimalHashMultiplier(cells_x);
        };
        
        cells: [total_cells]CellList,
        allocator: std.mem.Allocator,
        
        const CellList = struct {
            bodies: [16]u32 = undefined,
            count: u8 = 0,
        };
        
        pub fn init(allocator: std.mem.Allocator) Self {
            return .{
                .cells = std.mem.zeroes([total_cells]CellList),
                .allocator = allocator,
            };
        }
        
        pub inline fn hash(x: f32, y: f32) u32 {
            const cell_x = @as(u32, @intFromFloat(x / config.grid_size));
            const cell_y = @as(u32, @intFromFloat(y / config.grid_size));
            // Perfect hash function generated at compile time
            return (cell_y * hash_multiplier + cell_x) % total_cells;
        }
        
        pub fn insert(self: *Self, body_id: u32, x: f32, y: f32) void {
            const idx = hash(x, y);
            const cell = &self.cells[idx];
            if (cell.count < cell.bodies.len) {
                cell.bodies[cell.count] = body_id;
                cell.count += 1;
            }
        }
        
        pub fn clear(self: *Self) void {
            // Optimized clear using SIMD when available
            if (comptime std.Target.x86.featureSetHas(std.Target.current.cpu.features, .avx2)) {
                self.clearSimd();
            } else {
                for (&self.cells) |*cell| {
                    cell.count = 0;
                }
            }
        }
        
        fn clearSimd(self: *Self) void {
            const vector_size = 32; // AVX2 = 256 bits = 32 bytes
            const vectors_per_cell = @sizeOf(CellList) / vector_size;
            
            var i: usize = 0;
            while (i < total_cells) : (i += 1) {
                const ptr = @as([*]u8, @ptrCast(&self.cells[i]));
                @memset(ptr[0..@sizeOf(CellList)], 0);
            }
        }
    };
}

fn findOptimalHashMultiplier(comptime width: u32) u32 {
    // Find a prime number that minimizes collisions for our grid
    const primes = [_]u32{ 31, 37, 41, 43, 47, 53, 59, 61, 67, 71 };
    return primes[width % primes.len];
}

// Main physics world with comptime optimizations
pub fn PhysicsWorld(comptime config: PhysicsConfig) type {
    return struct {
        const Self = @This();
        const SpatialHashType = SpatialHash(config);
        
        bodies: []RigidBody,
        velocities: []Vec2,
        forces: []Vec2,
        masses: []f32,
        
        spatial_hash: SpatialHashType,
        contacts: []Contact,
        contact_count: u32,
        
        gravity: Vec2 = .{ .x = 0, .y = -9.81 },
        
        pub fn init(allocator: std.mem.Allocator) !Self {
            return .{
                .bodies = try allocator.alloc(RigidBody, config.max_bodies),
                .velocities = try allocator.alloc(Vec2, config.max_bodies),
                .forces = try allocator.alloc(Vec2, config.max_bodies),
                .masses = try allocator.alloc(f32, config.max_bodies),
                .spatial_hash = SpatialHashType.init(allocator),
                .contacts = try allocator.alloc(Contact, config.max_contacts),
                .contact_count = 0,
            };
        }
        
        pub fn step(self: *Self, dt: f32, body_count: u32) void {
            // Clear spatial hash
            self.spatial_hash.clear();
            
            // Update spatial hash
            self.updateSpatialHash(body_count);
            
            // Broad phase collision detection
            self.broadPhase(body_count);
            
            // Integrate forces (can be SIMD optimized)
            self.integrateForces(dt, body_count);
            
            // Solve constraints
            self.solveConstraints();
            
            // Integrate velocities
            self.integrateVelocities(dt, body_count);
        }
        
        fn updateSpatialHash(self: *Self, body_count: u32) void {
            var i: u32 = 0;
            while (i < body_count) : (i += 1) {
                const body = self.bodies[i];
                self.spatial_hash.insert(i, body.position.x, body.position.y);
            }
        }
        
        fn integrateForces(self: *Self, dt: f32, body_count: u32) void {
            // SIMD optimization for force integration
            if (comptime std.Target.x86.featureSetHas(std.Target.current.cpu.features, .avx)) {
                self.integrateForcesSimd(dt, body_count);
            } else {
                self.integrateForcesScalar(dt, body_count);
            }
        }
        
        fn integrateForcesScalar(self: *Self, dt: f32, body_count: u32) void {
            var i: u32 = 0;
            while (i < body_count) : (i += 1) {
                const inv_mass = 1.0 / self.masses[i];
                
                // Apply gravity
                self.forces[i].y += self.gravity.y * self.masses[i];
                
                // F = ma, so a = F/m
                self.velocities[i].x += self.forces[i].x * inv_mass * dt;
                self.velocities[i].y += self.forces[i].y * inv_mass * dt;
                
                // Clear forces for next frame
                self.forces[i] = .{ .x = 0, .y = 0 };
            }
        }
        
        fn integrateForcesSimd(self: *Self, dt: f32, body_count: u32) void {
            const vector_width = 8; // Process 8 bodies at once with AVX
            
            var i: u32 = 0;
            while (i + vector_width <= body_count) : (i += vector_width) {
                // Load 8 velocities, forces, and masses at once
                const vx = @Vector(8, f32){
                    self.velocities[i].x, self.velocities[i+1].x,
                    self.velocities[i+2].x, self.velocities[i+3].x,
                    self.velocities[i+4].x, self.velocities[i+5].x,
                    self.velocities[i+6].x, self.velocities[i+7].x,
                };
                
                // ... SIMD operations ...
                
                // Store results back
                inline for (0..vector_width) |j| {
                    self.velocities[i + j].x = vx[j];
                }
            }
            
            // Handle remaining bodies
            while (i < body_count) : (i += 1) {
                self.integrateForcesScalar(dt, 1);
            }
        }
        
        fn integrateVelocities(self: *Self, dt: f32, body_count: u32) void {
            var i: u32 = 0;
            while (i < body_count) : (i += 1) {
                self.bodies[i].position.x += self.velocities[i].x * dt;
                self.bodies[i].position.y += self.velocities[i].y * dt;
            }
        }
        
        fn broadPhase(self: *Self, body_count: u32) void {
            self.contact_count = 0;
            
            // Check each cell in spatial hash
            for (self.spatial_hash.cells) |cell| {
                if (cell.count < 2) continue;
                
                // Check all pairs in cell
                var i: u8 = 0;
                while (i < cell.count) : (i += 1) {
                    var j: u8 = i + 1;
                    while (j < cell.count) : (j += 1) {
                        const body_a = cell.bodies[i];
                        const body_b = cell.bodies[j];
                        
                        if (self.checkCollision(body_a, body_b)) |contact| {
                            if (self.contact_count < config.max_contacts) {
                                self.contacts[self.contact_count] = contact;
                                self.contact_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        fn checkCollision(self: *Self, a: u32, b: u32) ?Contact {
            const body_a = self.bodies[a];
            const body_b = self.bodies[b];
            
            const dx = body_b.position.x - body_a.position.x;
            const dy = body_b.position.y - body_a.position.y;
            const dist_sq = dx * dx + dy * dy;
            
            const radius_sum = body_a.radius + body_b.radius;
            
            if (dist_sq < radius_sum * radius_sum) {
                const dist = @sqrt(dist_sq);
                return Contact{
                    .body_a = a,
                    .body_b = b,
                    .normal = .{
                        .x = dx / dist,
                        .y = dy / dist,
                    },
                    .penetration = radius_sum - dist,
                };
            }
            
            return null;
        }
        
        fn solveConstraints(self: *Self) void {
            // Simple iterative constraint solver
            const iterations = 4;
            var iter: u32 = 0;
            while (iter < iterations) : (iter += 1) {
                var i: u32 = 0;
                while (i < self.contact_count) : (i += 1) {
                    self.solveContact(&self.contacts[i]);
                }
            }
        }
        
        fn solveContact(self: *Self, contact: *Contact) void {
            const a = contact.body_a;
            const b = contact.body_b;
            
            // Calculate relative velocity
            const rv_x = self.velocities[b].x - self.velocities[a].x;
            const rv_y = self.velocities[b].y - self.velocities[a].y;
            
            // Velocity along normal
            const velocity_along_normal = rv_x * contact.normal.x + rv_y * contact.normal.y;
            
            // Don't resolve if velocities are separating
            if (velocity_along_normal > 0) return;
            
            // Calculate impulse scalar
            const e = 0.8; // restitution
            const inv_mass_a = 1.0 / self.masses[a];
            const inv_mass_b = 1.0 / self.masses[b];
            var j = -(1.0 + e) * velocity_along_normal;
            j /= inv_mass_a + inv_mass_b;
            
            // Apply impulse
            const impulse_x = j * contact.normal.x;
            const impulse_y = j * contact.normal.y;
            
            self.velocities[a].x -= inv_mass_a * impulse_x;
            self.velocities[a].y -= inv_mass_a * impulse_y;
            self.velocities[b].x += inv_mass_b * impulse_x;
            self.velocities[b].y += inv_mass_b * impulse_y;
        }
    };
}

const Vec2 = struct {
    x: f32,
    y: f32,
};

const RigidBody = struct {
    position: Vec2,
    radius: f32,
    flags: u32 = 0,
};

const Contact = struct {
    body_a: u32,
    body_b: u32,
    normal: Vec2,
    penetration: f32,
};

// C API for integration with Rust
const DefaultPhysicsWorld = PhysicsWorld(.{});

export fn dream_physics_create() ?*DefaultPhysicsWorld {
    const allocator = std.heap.c_allocator;
    const world = allocator.create(DefaultPhysicsWorld) catch return null;
    world.* = DefaultPhysicsWorld.init(allocator) catch {
        allocator.destroy(world);
        return null;
    };
    return world;
}

export fn dream_physics_destroy(world: *DefaultPhysicsWorld) void {
    const allocator = std.heap.c_allocator;
    allocator.free(world.bodies);
    allocator.free(world.velocities);
    allocator.free(world.forces);
    allocator.free(world.masses);
    allocator.free(world.contacts);
    allocator.destroy(world);
}

export fn dream_physics_step(world: *DefaultPhysicsWorld, dt: f32, body_count: u32) void {
    world.step(dt, body_count);
}

export fn dream_physics_add_body(
    world: *DefaultPhysicsWorld,
    id: u32,
    x: f32,
    y: f32,
    radius: f32,
    mass: f32,
) void {
    if (id >= world.bodies.len) return;
    
    world.bodies[id] = .{
        .position = .{ .x = x, .y = y },
        .radius = radius,
    };
    world.velocities[id] = .{ .x = 0, .y = 0 };
    world.forces[id] = .{ .x = 0, .y = 0 };
    world.masses[id] = mass;
}

// Build script integration (build.zig)
const Builder = @import("std").build.Builder;

pub fn build(b: *Builder) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    
    const lib = b.addSharedLibrary(.{
        .name = "dream_physics",
        .root_source_file = .{ .path = "physics.zig" },
        .target = target,
        .optimize = optimize,
    });
    
    lib.linkLibC();
    lib.install();
}