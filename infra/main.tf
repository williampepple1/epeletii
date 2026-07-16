terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5"
    }
  }
}

provider "aws" {
  region  = var.aws_region
  profile = var.aws_profile
}

# Security group for the game server
resource "aws_security_group" "game_server" {
  name        = "epeletii-game-server"
  description = "Allow WebSocket (9001) and SSH from trusted sources"

  ingress {
    description = "WebSocket game server"
    from_port   = 9001
    to_port     = 9001
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "SSH from your IP"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = var.ssh_allowed_cidrs
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = { Name = "epeletii" }
}

# Elastic IP for the game server
resource "aws_eip" "game_server" {
  domain = "vpc"
  tags   = { Name = "epeletii" }
}

# EC2 instance (x86 — no cross-compilation needed)
resource "aws_instance" "game_server" {
  ami                    = data.aws_ami.ubuntu_amd64.id
  instance_type          = var.instance_type
  vpc_security_group_ids = [aws_security_group.game_server.id]

  associate_public_ip_address = false # we use EIP
  key_name                    = var.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecr_pull.name

  user_data = templatefile("${path.module}/user-data.sh", {
    docker_image = var.docker_image
    mongo_uri    = var.mongo_uri
    jwt_secret   = var.jwt_secret
  })

  root_block_device {
    volume_size = 8
    volume_type = "gp3"
    encrypted   = true
  }

  tags = { Name = "epeletii" }
}

# Attach Elastic IP
resource "aws_eip_association" "game_server" {
  instance_id   = aws_instance.game_server.id
  allocation_id = aws_eip.game_server.id
}

# IAM role allowing EC2 to pull from ECR (optional — switch to Docker Hub and remove)
resource "aws_iam_role" "ec2_ecr" {
  name = "epeletii-ec2-ecr"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Service = "ec2.amazonaws.com" }
      Action = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_instance_profile" "ecr_pull" {
  name = "epeletii-ecr-pull"
  role = aws_iam_role.ec2_ecr.name
}

# AMI lookup
data "aws_ami" "ubuntu_amd64" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd-gp3/ubuntu-24.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}
