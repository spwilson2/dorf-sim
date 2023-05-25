@tool
extends EditorPlugin


func _enter_tree():
	pass


func _exit_tree():
	pass

func _build():
	return OS.execute("/usr/bin/env", ["python3", "addons/protobuilder/build.py"]) == 0

func _apply_changes():
	if _build() != 0:
		push_error("Build failed")