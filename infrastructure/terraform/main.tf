# ERP System - Terraform Main Configuration
# Deploys complete ERP infrastructure on AWS

terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.20"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.10"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.4"
    }
  }

  backend "s3" {
    # Configure in terraform.tfvars or environment
    # bucket = "your-terraform-state-bucket"
    # key    = "erp-system/terraform.tfstate"
    # region = "us-west-2"
  }
}

# Configure AWS Provider
provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "ERP-System"
      Environment = var.environment
      ManagedBy   = "Terraform"
      Owner       = var.owner
    }
  }
}

# Data sources
data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}

# Local values
locals {
  cluster_name = "${var.project_name}-${var.environment}"
  common_tags = {
    Project     = var.project_name
    Environment = var.environment
    ManagedBy   = "Terraform"
    Owner       = var.owner
  }
}

# VPC Module
module "vpc" {
  source = "./modules/vpc"

  project_name         = var.project_name
  environment          = var.environment
  vpc_cidr            = var.vpc_cidr
  availability_zones  = data.aws_availability_zones.available.names
  enable_nat_gateway  = var.enable_nat_gateway
  enable_vpn_gateway  = var.enable_vpn_gateway

  tags = local.common_tags
}

# Security Groups Module
module "security_groups" {
  source = "./modules/security"

  project_name = var.project_name
  environment  = var.environment
  vpc_id       = module.vpc.vpc_id

  tags = local.common_tags
}

# EKS Cluster Module
module "eks" {
  source = "./modules/eks"

  cluster_name       = local.cluster_name
  kubernetes_version = var.kubernetes_version
  vpc_id             = module.vpc.vpc_id
  subnet_ids         = module.vpc.private_subnet_ids
  security_group_ids = [module.security_groups.eks_security_group_id]

  node_groups = var.eks_node_groups

  tags = local.common_tags
}

# RDS PostgreSQL Module
module "rds" {
  source = "./modules/rds"

  project_name        = var.project_name
  environment         = var.environment
  vpc_id              = module.vpc.vpc_id
  subnet_ids          = module.vpc.database_subnet_ids
  security_group_ids  = [module.security_groups.rds_security_group_id]

  db_name                = var.db_name
  db_username            = var.db_username
  db_password            = var.db_password
  db_instance_class      = var.db_instance_class
  db_allocated_storage   = var.db_allocated_storage
  db_engine_version      = var.db_engine_version
  backup_retention_period = var.backup_retention_period
  backup_window          = var.backup_window
  maintenance_window     = var.maintenance_window

  monitoring_enabled     = var.enable_monitoring
  performance_insights   = var.enable_performance_insights
  multi_az              = var.multi_az_deployment

  tags = local.common_tags
}

# ElastiCache Redis Module
module "redis" {
  source = "./modules/redis"

  project_name       = var.project_name
  environment        = var.environment
  vpc_id             = module.vpc.vpc_id
  subnet_ids         = module.vpc.private_subnet_ids
  security_group_ids = [module.security_groups.redis_security_group_id]

  node_type           = var.redis_node_type
  num_cache_nodes     = var.redis_num_nodes
  parameter_group     = var.redis_parameter_group
  engine_version      = var.redis_engine_version
  port                = var.redis_port

  tags = local.common_tags
}

# Application Load Balancer Module
module "alb" {
  source = "./modules/alb"

  project_name       = var.project_name
  environment        = var.environment
  vpc_id             = module.vpc.vpc_id
  subnet_ids         = module.vpc.public_subnet_ids
  security_group_ids = [module.security_groups.alb_security_group_id]

  certificate_arn = var.ssl_certificate_arn
  domain_name     = var.domain_name

  tags = local.common_tags
}

# S3 Buckets Module
module "s3" {
  source = "./modules/s3"

  project_name = var.project_name
  environment  = var.environment

  enable_versioning = var.s3_enable_versioning
  enable_encryption = var.s3_enable_encryption
  lifecycle_rules   = var.s3_lifecycle_rules

  tags = local.common_tags
}

# CloudWatch & Monitoring Module
module "monitoring" {
  source = "./modules/monitoring"

  project_name  = var.project_name
  environment   = var.environment
  cluster_name  = local.cluster_name

  enable_container_insights = var.enable_container_insights
  log_retention_days       = var.log_retention_days

  # SNS topic for alerts
  alert_email = var.alert_email

  tags = local.common_tags
}

# Backup Module
module "backup" {
  source = "./modules/backup"

  project_name = var.project_name
  environment  = var.environment

  backup_vault_name = "${var.project_name}-${var.environment}-backup-vault"
  backup_schedule   = var.backup_schedule
  backup_retention  = var.backup_retention

  # Resources to backup
  rds_arn           = module.rds.db_instance_arn
  s3_bucket_arns    = module.s3.bucket_arns

  tags = local.common_tags
}

# Secrets Manager
resource "aws_secretsmanager_secret" "app_secrets" {
  name                    = "${var.project_name}-${var.environment}-app-secrets"
  description             = "Application secrets for ERP system"
  recovery_window_in_days = var.environment == "production" ? 30 : 0

  tags = local.common_tags
}

resource "aws_secretsmanager_secret_version" "app_secrets" {
  secret_id = aws_secretsmanager_secret.app_secrets.id
  secret_string = jsonencode({
    database_url        = "postgresql://${var.db_username}:${var.db_password}@${module.rds.db_endpoint}:5432/${var.db_name}"
    redis_url          = "redis://${module.redis.primary_endpoint}:${var.redis_port}"
    jwt_secret         = random_password.jwt_secret.result
    encryption_key     = random_password.encryption_key.result
  })
}

# Generate random secrets
resource "random_password" "jwt_secret" {
  length  = 64
  special = true
}

resource "random_password" "encryption_key" {
  length  = 32
  special = false
}

# ECR Repository for container images
resource "aws_ecr_repository" "erp_api" {
  name                 = "${var.project_name}-api"
  image_tag_mutability = "MUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }

  encryption_configuration {
    encryption_type = "AES256"
  }

  tags = local.common_tags
}

# ECR Lifecycle Policy
resource "aws_ecr_lifecycle_policy" "erp_api" {
  repository = aws_ecr_repository.erp_api.name

  policy = jsonencode({
    rules = [
      {
        rulePriority = 1
        description  = "Keep last 10 production images"
        selection = {
          tagStatus     = "tagged"
          tagPrefixList = ["prod"]
          countType     = "imageCountMoreThan"
          countNumber   = 10
        }
        action = {
          type = "expire"
        }
      },
      {
        rulePriority = 2
        description  = "Keep last 5 staging images"
        selection = {
          tagStatus     = "tagged"
          tagPrefixList = ["staging"]
          countType     = "imageCountMoreThan"
          countNumber   = 5
        }
        action = {
          type = "expire"
        }
      },
      {
        rulePriority = 3
        description  = "Delete untagged images older than 1 day"
        selection = {
          tagStatus   = "untagged"
          countType   = "sinceImagePushed"
          countUnit   = "days"
          countNumber = 1
        }
        action = {
          type = "expire"
        }
      }
    ]
  })
}

# Route53 DNS (if domain is provided)
resource "aws_route53_zone" "main" {
  count = var.domain_name != "" ? 1 : 0
  name  = var.domain_name

  tags = local.common_tags
}

resource "aws_route53_record" "api" {
  count   = var.domain_name != "" ? 1 : 0
  zone_id = aws_route53_zone.main[0].zone_id
  name    = "api.${var.domain_name}"
  type    = "A"

  alias {
    name                   = module.alb.dns_name
    zone_id                = module.alb.zone_id
    evaluate_target_health = true
  }
}

# Output important values
output "cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "database_endpoint" {
  description = "RDS instance endpoint"
  value       = module.rds.db_endpoint
  sensitive   = true
}

output "redis_endpoint" {
  description = "ElastiCache Redis endpoint"
  value       = module.redis.primary_endpoint
  sensitive   = true
}

output "load_balancer_dns" {
  description = "Application Load Balancer DNS name"
  value       = module.alb.dns_name
}

output "ecr_repository_url" {
  description = "ECR repository URL"
  value       = aws_ecr_repository.erp_api.repository_url
}

output "secrets_manager_arn" {
  description = "Secrets Manager ARN"
  value       = aws_secretsmanager_secret.app_secrets.arn
}

output "vpc_id" {
  description = "VPC ID"
  value       = module.vpc.vpc_id
}

output "private_subnet_ids" {
  description = "Private subnet IDs"
  value       = module.vpc.private_subnet_ids
}

output "s3_bucket_names" {
  description = "S3 bucket names"
  value       = module.s3.bucket_names
}