{
  "widgets": [
    {
      "type": "metric", "x": 0, "y": 0, "width": 12, "height": 6,
      "properties": {
        "title": "Quote: request/phút", "view": "timeSeries", "region": "__REGION__", "period": 60, "stat": "Sum",
        "metrics": [["AWS/ApplicationELB", "RequestCount", "LoadBalancer", "__LB__"]]
      }
    },
    {
      "type": "metric", "x": 12, "y": 0, "width": 12, "height": 6,
      "properties": {
        "title": "Quote: latency p95 (giây)", "view": "timeSeries", "region": "__REGION__", "period": 60, "stat": "p95",
        "metrics": [["AWS/ApplicationELB", "TargetResponseTime", "LoadBalancer", "__LB__"]]
      }
    },
    {
      "type": "metric", "x": 0, "y": 6, "width": 12, "height": 6,
      "properties": {
        "title": "Quote: lỗi 5xx", "view": "timeSeries", "region": "__REGION__", "period": 60, "stat": "Sum",
        "metrics": [
          ["AWS/ApplicationELB", "HTTPCode_Target_5XX_Count", "LoadBalancer", "__LB__"],
          ["AWS/ApplicationELB", "HTTPCode_ELB_5XX_Count", "LoadBalancer", "__LB__"]
        ]
      }
    },
    {
      "type": "metric", "x": 12, "y": 6, "width": 12, "height": 6,
      "properties": {
        "title": "Target khoẻ / không khoẻ", "view": "timeSeries", "region": "__REGION__", "period": 60, "stat": "Average",
        "metrics": [
          ["AWS/ApplicationELB", "HealthyHostCount", "TargetGroup", "__TG__", "LoadBalancer", "__LB__"],
          ["AWS/ApplicationELB", "UnHealthyHostCount", "TargetGroup", "__TG__", "LoadBalancer", "__LB__"]
        ]
      }
    },
    {
      "type": "metric", "x": 0, "y": 12, "width": 12, "height": 6,
      "properties": {
        "title": "CPU của ASG (%)", "view": "timeSeries", "region": "__REGION__", "period": 60, "stat": "Average",
        "metrics": [["AWS/EC2", "CPUUtilization", "AutoScalingGroupName", "__ASG__"]]
      }
    },
    {
      "type": "metric", "x": 12, "y": 12, "width": 12, "height": 6,
      "properties": {
        "title": "RDS: DatabaseConnections", "view": "timeSeries", "region": "__REGION__", "period": 60, "stat": "Maximum",
        "metrics": [["AWS/RDS", "DatabaseConnections", "DBInstanceIdentifier", "__DB__"]]
      }
    }
  ]
}
