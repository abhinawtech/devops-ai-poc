terraform {
  required_version = ">= 1.0"
  
  backend "s3" {
    bucket = "tfstate-620173142956-us-west-2"  # Change this
    key    = "ai-model-service/dev/terraform.tfstate"
    region = "us-west-2"
  }
}

provider "aws" {
  region = "us-west-2"
}

module "eks_cluster" {
  source = "../../modules/eks-cluster"
  
  cluster_name         = "ai-model-dev"
  environment          = "dev"
  node_instance_types  = ["t3.small"]
  min_nodes           = 1
  max_nodes           = 3
  desired_nodes       = 1
}