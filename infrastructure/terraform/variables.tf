# ERP System - Terraform Variables

# General Configuration
variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "erp-system"
}

variable "environment" {
  description = "Environment name (dev, staging, production)"
  type        = string
  validation {
    condition     = contains(["dev", "staging", "production"], var.environment)
    error_message = "Environment must be one of: dev, staging, production."
  }
}

variable "owner" {
  description = "Owner of the resources"
  type        = string
  default     = "ERP-Team"
}

variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-west-2"
}

# Networking Configuration
variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "enable_nat_gateway" {
  description = "Enable NAT Gateway for private subnets"
  type        = bool
  default     = true
}

variable "enable_vpn_gateway" {
  description = "Enable VPN Gateway"
  type        = bool
  default     = false
}

# EKS Configuration
variable "kubernetes_version" {
  description = "Kubernetes version for EKS cluster"
  type        = string
  default     = "1.28"
}

variable "eks_node_groups" {
  description = "EKS node group configurations"
  type = map(object({
    instance_types = list(string)
    capacity_type  = string
    scaling_config = object({
      desired_size = number
      max_size     = number
      min_size     = number
    })
    update_config = object({
      max_unavailable_percentage = number
    })
  }))
  default = {
    main = {
      instance_types = ["t3.medium"]
      capacity_type  = "ON_DEMAND"
      scaling_config = {
        desired_size = 2
        max_size     = 10
        min_size     = 1
      }
      update_config = {
        max_unavailable_percentage = 25
      }
    }
  }
}

# Database Configuration
variable "db_name" {
  description = "Name of the RDS database"
  type        = string
  default     = "erp_main"
}

variable "db_username" {
  description = "Username for the RDS database"
  type        = string
  default     = "erp_admin"
}

variable "db_password" {
  description = "Password for the RDS database"
  type        = string
  sensitive   = true
}

variable "db_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "db_allocated_storage" {
  description = "Allocated storage for RDS instance (GB)"
  type        = number
  default     = 100
}

variable "db_engine_version" {
  description = "PostgreSQL engine version"
  type        = string
  default     = "16.1"
}

variable "backup_retention_period" {
  description = "Backup retention period (days)"
  type        = number
  default     = 7
}

variable "backup_window" {
  description = "Backup window"
  type        = string
  default     = "07:00-09:00"
}

variable "maintenance_window" {
  description = "Maintenance window"
  type        = string
  default     = "sun:05:00-sun:07:00"
}

variable "multi_az_deployment" {
  description = "Enable Multi-AZ deployment for RDS"
  type        = bool
  default     = false
}

variable "enable_performance_insights" {
  description = "Enable Performance Insights for RDS"
  type        = bool
  default     = true
}

# Redis Configuration
variable "redis_node_type" {
  description = "ElastiCache Redis node type"
  type        = string
  default     = "cache.t3.micro"
}

variable "redis_num_nodes" {
  description = "Number of Redis nodes"
  type        = number
  default     = 1
}

variable "redis_parameter_group" {
  description = "Redis parameter group"
  type        = string
  default     = "default.redis7"
}

variable "redis_engine_version" {
  description = "Redis engine version"
  type        = string
  default     = "7.0"
}

variable "redis_port" {
  description = "Redis port"
  type        = number
  default     = 6379
}

# Application Configuration
variable "domain_name" {
  description = "Domain name for the application (leave empty to skip DNS setup)"
  type        = string
  default     = ""
}

variable "ssl_certificate_arn" {
  description = "ARN of SSL certificate in ACM (leave empty to skip HTTPS setup)"
  type        = string
  default     = ""
}

# S3 Configuration
variable "s3_enable_versioning" {
  description = "Enable versioning for S3 buckets"
  type        = bool
  default     = true
}

variable "s3_enable_encryption" {
  description = "Enable encryption for S3 buckets"
  type        = bool
  default     = true
}

variable "s3_lifecycle_rules" {
  description = "S3 lifecycle rules"
  type = list(object({
    id     = string
    status = string
    transitions = list(object({
      days          = number
      storage_class = string
    }))
    expiration = object({
      days = number
    })
  }))
  default = [
    {
      id     = "transition_to_ia"
      status = "Enabled"
      transitions = [
        {
          days          = 30
          storage_class = "STANDARD_IA"
        },
        {
          days          = 90
          storage_class = "GLACIER"
        }
      ]
      expiration = {
        days = 365
      }
    }
  ]
}

# Monitoring Configuration
variable "enable_monitoring" {
  description = "Enable detailed monitoring"
  type        = bool
  default     = true
}

variable "enable_container_insights" {
  description = "Enable EKS Container Insights"
  type        = bool
  default     = true
}

variable "log_retention_days" {
  description = "CloudWatch log retention period (days)"
  type        = number
  default     = 14
}

variable "alert_email" {
  description = "Email address for alerts"
  type        = string
  default     = ""
}

# Backup Configuration
variable "backup_schedule" {
  description = "Backup schedule (cron expression)"
  type        = string
  default     = "cron(0 2 * * ? *)" # Daily at 2 AM
}

variable "backup_retention" {
  description = "Backup retention configuration"
  type = object({
    cold_storage_after = number
    delete_after       = number
  })
  default = {
    cold_storage_after = 30
    delete_after       = 365
  }
}

# Feature Flags
variable "enable_spot_instances" {
  description = "Enable spot instances for EKS node groups"
  type        = bool
  default     = false
}

variable "enable_autoscaling" {
  description = "Enable cluster autoscaling"
  type        = bool
  default     = true
}

variable "enable_pod_security" {
  description = "Enable Pod Security Standards"
  type        = bool
  default     = true
}

variable "enable_network_policy" {
  description = "Enable Kubernetes Network Policies"
  type        = bool
  default     = true
}

# Cost Optimization
variable "enable_cost_optimization" {
  description = "Enable cost optimization features"
  type        = bool
  default     = true
}

variable "enable_scheduled_scaling" {
  description = "Enable scheduled scaling for cost optimization"
  type        = bool
  default     = false
}

variable "scheduled_scaling_config" {
  description = "Scheduled scaling configuration"
  type = object({
    scale_down_schedule = string
    scale_up_schedule   = string
    min_size_scaled     = number
  })
  default = {
    scale_down_schedule = "0 19 * * MON-FRI" # 7 PM on weekdays
    scale_up_schedule   = "0 8 * * MON-FRI"  # 8 AM on weekdays
    min_size_scaled     = 0
  }
}

# Security Configuration
variable "enable_encryption_at_rest" {
  description = "Enable encryption at rest for all supported services"
  type        = bool
  default     = true
}

variable "enable_encryption_in_transit" {
  description = "Enable encryption in transit"
  type        = bool
  default     = true
}

variable "enable_secrets_manager" {
  description = "Use AWS Secrets Manager for sensitive data"
  type        = bool
  default     = true
}

variable "allowed_cidr_blocks" {
  description = "CIDR blocks allowed to access the ERP system"
  type        = list(string)
  default     = ["0.0.0.0/0"] # Restrict this in production
}

# Compliance
variable "enable_compliance_logging" {
  description = "Enable compliance logging features"
  type        = bool
  default     = true
}

variable "enable_data_residency" {
  description = "Enable data residency controls"
  type        = bool
  default     = false
}

variable "compliance_standards" {
  description = "Compliance standards to adhere to"
  type        = list(string)
  default     = ["SOC2", "ISO27001"]
}