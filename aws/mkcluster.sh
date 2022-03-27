source ../.env

CLUSTER_NAME=
SUBNET_IDS=
SECURITY_GROUP_IDS=
ROLE_ARN=

aws eks create-cluster \
   --region $AWS_REGION \
   --name $CLUSTER_NAME \
   --kubernetes-version 1.21 \
   --role-arn $ROLE_ARN \
   --resources-vpc-config subnetIds=$SUBNET_IDS,securityGroupIds=$SECURITY_GROUP_IDS
