// Zig build configuration

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    
    const physics_lib = b.addSharedLibrary(.{
        .name = "dream_physics",
        .root_source_file = .{ .path = "physics.zig" },
        .target = target,
        .optimize = optimize,
    });
    
    physics_lib.linkLibC();
    b.installArtifact(physics_lib);
}
