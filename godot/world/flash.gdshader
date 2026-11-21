// Blends a shape towards white by `flash_amount`, for the white flash when a blow lands.
// A shader rather than a colour change, so it keeps working when placeholders become art.
shader_type canvas_item;

uniform float flash_amount : hint_range(0.0, 1.0) = 0.0;

void fragment() {
	COLOR.rgb = mix(COLOR.rgb, vec3(1.0), flash_amount);
}
