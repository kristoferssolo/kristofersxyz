from django.urls import path

from . import views

urlpatterns = [
    path("", views.projects, name="projects"),
    path("karbs", views.karbs, name="karbs"),
    path("karbs/instructions", views.instructions, name="instructions"),
    path("traffic-light-detector", views.traffic_light_detector, name="traffic-light-detector"),
]
