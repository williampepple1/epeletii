output "server_ip" {
  description = "Public IP of the game server"
  value       = aws_eip.game_server.public_ip
}

output "ws_url" {
  description = "WebSocket URL for the frontend"
  value       = "ws://${aws_eip.game_server.public_ip}:9001"
}

output "ssh_command" {
  description = "SSH command to access the server"
  value       = "ssh -i your-key.pem ubuntu@${aws_eip.game_server.public_ip}"
}
