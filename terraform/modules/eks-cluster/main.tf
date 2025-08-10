terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }
}

# Availability Zones (limit to 3 for subnets/NAT spread)
data "aws_availability_zones" "available" {
  filter {
    name   = "opt-in-status"
    values = ["opt-in-not-required"]
  }
}

# -------------------------
# VPC for the EKS cluster
# -------------------------
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "6.0.1"

  name = "${var.cluster_name}-vpc"
  cidr = var.vpc_cidr

  azs             = slice(data.aws_availability_zones.available.names, 0, 3)
  private_subnets = var.private_subnet_cidrs
  public_subnets  = var.public_subnet_cidrs

  enable_nat_gateway = true
  single_nat_gateway = var.environment == "dev"

  enable_dns_hostnames = true
  enable_dns_support   = true

  # Required for k8s LoadBalancer Services to place ELBs correctly
  private_subnet_tags = {
    "kubernetes.io/role/internal-elb"           = "1"
    "kubernetes.io/cluster/${var.cluster_name}" = "shared"
  }
  public_subnet_tags = {
    "kubernetes.io/role/elb"                    = "1"
    "kubernetes.io/cluster/${var.cluster_name}" = "shared"
  }

  tags = {
    Environment = var.environment
    Project     = "ai-model-service"
  }
}

# -------------------------
# EKS cluster + MNG
# -------------------------
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "21.0.8"

  # v21 input names
  name               = var.cluster_name
  kubernetes_version = var.kubernetes_version

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  # Public endpoint (lock this down for prod)
  endpoint_public_access       = true
  endpoint_public_access_cidrs = var.approved_cidrs

  # Let AWS manage core add-ons
  addons = {
    coredns    = {}
    kube-proxy = {}
    vpc-cni    = {}
  }

  # Make the creator admin via access entry
  enable_cluster_creator_admin_permissions = true

  # EKS Managed Node Group
  eks_managed_node_groups = {
    main = {
      name           = "${var.cluster_name}-nodes"
      instance_types = var.node_instance_types
      min_size       = var.min_nodes
      max_size       = var.max_nodes
      desired_size   = var.desired_nodes

      labels = {
        Environment = var.environment
        NodeGroup   = "main"
      }
    }
  }

  tags = {
    Environment = var.environment
    Project     = "ai-model-service"
  }
}
