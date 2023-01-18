from django.urls import path

from . import views

urlpatterns = [
    path("", views.index, name="home"),
    path("projects", views.projects, name="projects"),
    path("lightsaber", views.lightsaber, name="lightsaber"),
]
